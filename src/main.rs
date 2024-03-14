use std::env;

use anyhow::{anyhow, Result};
use mlua::prelude::*;

extern crate udev;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let script = std::fs::read_to_string(
        args.get(1).map(String::as_str).unwrap_or("/usr/lib/pinch/init.luau")
    ).map_err(|e| anyhow!("Failed to read file: {}", e))?;

    let lua = Lua::new();

    let map_table = lua.create_table()?;
    map_table.set(1, "one")?;
    map_table.set("two", 2)?;

    lua.globals().set("map_table", map_table)?;

    lua.load(script).exec()?;

    let mut enumerator = udev::Enumerator::new().unwrap();

    enumerator.match_subsystem("tty").unwrap();

    for device in enumerator.scan_devices().unwrap() {
        println!("found device: {:?}", device.syspath());
    }

    Ok(())
}
