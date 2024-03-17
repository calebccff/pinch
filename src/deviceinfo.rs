
use std::path::Path;
use mlua::prelude::*;

#[derive(Debug)]
pub struct Deviceinfo {
    name: String,
    super_partition: Option<String>,
    no_framebuffer: bool,
}

impl Deviceinfo {
    pub fn parse(path: &Path) -> Self {
        let contents = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                println!("Failed to read deviceinfo file: {} {}", path.display(), e.to_string());
                return Deviceinfo {
                    name: "Unknown".to_string(),
                    super_partition: None,
                    no_framebuffer: false,
                }
            }
        };

        let mut name: String = "Unknown".to_string();
        let mut super_partition: Option<String> = None;
        let mut no_framebuffer: bool = false;

        for line in contents.lines() {
            if !line.starts_with("deviceinfo_") {
                continue;
            }
            let mut parts = line.splitn(2, '=');
            let key = match parts.next() {
                Some(k) => k,
                None => continue,
            };
            let value = match parts.next() {
                Some(v) => v.trim().replace("\"", ""),
                None => continue,
            };

            match key {
                "deviceinfo_name" => name = value,
                "deviceinfo_no_framebuffer" => no_framebuffer = value.parse().unwrap_or(false),
                "super_partition" => super_partition = Some(value),
                _ => {}
            }
        }

        Deviceinfo {
            name,
            super_partition,
            no_framebuffer,
        }
    }
}

impl LuaUserData for Deviceinfo {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method_mut(LuaMetaMethod::NewIndex, |_, _, (key, _): (String, bool)| {
            let err: Result<(), LuaError> = Err(LuaError::external(format!("[deviceinfo] assinging to deviceinfo is disallowed! {}", key)));
            err
        });

        methods.add_meta_method(LuaMetaMethod::Index, |_, this, key: String| {
            match key.as_str() {
                "name" => Ok(this.name.clone()),
                "no_framebuffer" => Ok(this.no_framebuffer.to_string()),
                "super_partition" => Ok(this.super_partition.clone().unwrap_or("".into())),
                _ => Err(LuaError::external(format!("[deviceinfo] unknown key: {}", key))),
            }
        })
    }
}
