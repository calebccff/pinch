use std::collections::HashSet;
use std::env;
use std::path;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use mlua::prelude::*;
use glob;
use pinch::deviceinfo::Deviceinfo;
use xshell::{Shell, cmd};
use udev;

fn main() -> Result<()> {
    println!("[Starting Pinch]");
    let path = path::PathBuf::from(env::args().skip(1).next().unwrap_or("/usr/lib/pinch".into()));
    let is_sandbox = env::args().any(|x| x == "--sandbox");

    let lua = Lua::new();

    /* Initialise early globals */
    lua_early_globals(&lua, is_sandbox)?;

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
            Ok(PinchUdev { udev})
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

    let write_sysfs = lua.create_function(|_, (path, contents) : (String, String)| {
        println!("-- {} > {}", path, contents);
        std::fs::write(path, contents)?;
        Ok(())
    })?;

    lua.globals().set("write_sysfs", write_sysfs)?;

    Ok(())
}

fn lua_early_globals(lua: &Lua, is_sandbox: bool) -> Result<()> {
    let deviceinfo_path = path::PathBuf::from(env::args().skip(2).next().unwrap_or("/usr/share/deviceinfo/deviceinfo".into()));

    lua.globals().set("kernel", Kernel { probe_modules: HashSet::new() })?;
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

#[derive(Clone)]
struct Kernel {
    probe_modules: HashSet<String>,
}

impl LuaUserData for Kernel {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("probe", |_, this, ()| {
            println!("[RUST]: probing modules: {:?}", this.probe_modules);
            let sh = Shell::new().map_err(|e| LuaError::external(e.to_string()))?;
            for module in &this.probe_modules {
                cmd!(sh, "modprobe {module}").run().map_err(|e| LuaError::external(e.to_string()))?
            }
            Ok(())
        });
    }

    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("probe_modules", |_, this| {
            Ok(this.probe_modules.clone())
        });

        fields.add_field_method_set("probe_modules", |_, this, modules: Vec<String>| {
            this.probe_modules.extend(modules);
            Ok(())
        });
    }
}

#[derive(Clone)]
struct PinchUdev {
    udev: udev::Enumerator
}

impl LuaUserData for PinchUdev {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("match_subsystem", |_, this, subsystem: String| {
            println!("[RUST]: udev: matching subsystem: {}", subsystem);
            this.udev.match_subsystem(&subsystem).map_err(|e| LuaError::external(e.to_string()))
        });

        methods.add_method_mut("scan", |_, this, ()| {
            println!("[RUST]: udev: scanning devices");
            this.udev.scan_devices().map_err(|e| LuaError::external(e.to_string()))
            /* Map into the PinchDevice wrapper for Lua. Filter out all devices that don't have a devnode */
                .map(|devices| devices.filter_map(|d| match d.devnode() {
                    Some(_) => Some(PinchDevice(d)),
                    None => None
            }).collect::<Vec<_>>())
        });
    }
}

#[derive(Clone)]
struct PinchDevice(udev::Device);

impl LuaUserData for PinchDevice {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("syspath", |_, this| {
            Ok(this.0.syspath().to_string_lossy().into_owned())
        });

        fields.add_field_method_get("devnode", |_, this| {
            Ok(this.0.devnode().and_then(|x| Some(x.to_string_lossy().into_owned())).unwrap_or("".into()))
        });

        fields.add_field_method_get("subsystem", |_, this| {
            Ok(this.0.subsystem().and_then(|x| Some(x.to_string_lossy().into_owned())).unwrap_or("".into()))
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
            Ok(format!("{:24} {:16} {}",
                match this.0.devnode() { Some(x) => x.to_string_lossy().into_owned(), None => "".into()},
                match this.0.subsystem() { Some(x) => x.to_string_lossy().into_owned(), None => "".into()},
                this.0.syspath().to_string_lossy().into_owned()))
        });
    }
}
