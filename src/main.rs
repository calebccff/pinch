use std::collections::HashSet;
use std::env;
use std::path;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use mlua::prelude::*;
use glob;
use pinch::deviceinfo::Deviceinfo;
use pinch::kernel::Kernel;
use pinch::uevent::{PinchUdev, PinchDevice};
use xshell::{Shell, cmd};
use udev;

fn main() -> Result<()> {
    println!("[Starting Pinch]");
    let path = path::PathBuf::from(env::args().skip(1).next().unwrap_or("/usr/lib/pinch".into()));
    let is_sandbox = env::args().any(|x| x == "--sandbox");

    let lua = Lua::new();

    /* Initialise early globals */
    lua_early_globals(&lua, is_sandbox)?;

    //println!("globals: {:#?}", lua.globals());

    env::set_current_dir(&path)?;

    /* Parse configs */
    println!("-- Initializing Lua environment");
    lua_exec(&lua, PathBuf::from("init.env.lua"))?;
    for path in glob::glob("/usr/share/pinch/*.lua")? {
        if let Ok(script) = path {
            lua_exec(&lua, script)?;
        }
    }

    /* Initialise runtime globals */
    lua_runtime_globals(&lua)?;

    /* Run Init! */
    lua_exec(&lua, PathBuf::from("init.lua"))?;

    Ok(())
}

fn lua_runtime_globals(lua: &Lua) -> Result<()> {
    let udev_constructor = lua.create_function(
        |_, ()| {
            let udev = udev::Enumerator::new()?;
            //udev.match_is_initialized()?;
            Ok(PinchUdev::new(udev))
    })?;

    lua.globals().set("Udev", udev_constructor)?;

    let run_cmd = lua.create_function(|_, exec: String| {
        let sh = Shell::new().map_err(|e| LuaError::external(e.to_string()))?;
        let (program, args) = exec.split_once(' ').unwrap_or(("", ""));
        sh.cmd(&program).args(args.split(' ')) /* .quiet() */
            .run().map_err(|e| LuaError::external(e.to_string()))?;
        Ok(())
    })?;

    lua.globals().set("run", run_cmd)?;

    let write = lua.create_function(|_, (path, contents) : (String, String)| {
        println!("[WRITE] {} -> {}", contents, path);
        std::fs::write(path, contents)?;
        Ok(())
    })?;

    lua.globals().set("write", write)?;

    Ok(())
}

fn lua_early_globals(lua: &Lua, is_sandbox: bool) -> Result<()> {
    let deviceinfo_path = path::PathBuf::from(env::args().skip(2).next().unwrap_or("/usr/share/deviceinfo/deviceinfo".into()));

    lua.globals().set("kernel", Kernel::default())?;
    lua.globals().set("deviceinfo", Deviceinfo::parse(&deviceinfo_path))?;
    lua.globals().set("sandbox", is_sandbox)?;

    /* Override the print function to add our prefix */
    let print = lua.create_function(|_, line: String |{
        println!("[LUA] {}", line);
        Ok(())
    })?;

    lua.globals().set("print", print)?;
    Ok(())
}

fn lua_exec(lua: &Lua, path: PathBuf) -> Result<()> {
    println!("-- Running script: {}", path.display());
    let script = std::fs::read_to_string(path).map_err(|e| anyhow!("Failed to read file: {}", e))?;
    lua.load(script).exec()?;
    Ok(())
}
