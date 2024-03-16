use std::env;

use anyhow::{anyhow, Result};
use mlua::prelude::*;

extern crate udev;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let script = std::fs::read_to_string(
        args.get(1).map(String::as_str).unwrap_or("/usr/lib/pinch/init.lua")
    ).map_err(|e| anyhow!("Failed to read file: {}", e))?;

    env::set_current_dir("/usr/lib/pinch")?;

    let lua = Lua::new();

    let map_table = lua.create_table()?;
    map_table.set(1, "one")?;
    map_table.set("two", 2)?;

    lua.globals().set("map_table", map_table)?;

    lua.load(script).exec()?;

    let mut enumerator = udev::Enumerator::new()?;

    enumerator.match_is_initialized()?;

    for device in enumerator.scan_devices()? {
        println!("found device: {:?}", device.syspath());
    }

    Ok(())
}
