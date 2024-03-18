use std::{collections::HashSet, path::Path};
use mlua::prelude::*;
use xshell::{cmd, Shell};
use sys_mount::*;

//#[derive(Clone)]
pub struct Kernel {
    probe_modules: HashSet<String>,
    mountpoints: Vec<Mount>,
}

impl Default for Kernel {
    fn default() -> Self {
        Kernel {
            probe_modules: HashSet::new(),
            mountpoints: Vec::new(),
        }
    }

}

impl LuaUserData for Kernel {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("probe", |_, this, ()| {
            //println!("[KERNEL]: probing modules: {:?}", this.probe_modules);
            let sh = Shell::new().map_err(|e| LuaError::external(e.to_string()))?;
            for module in &this.probe_modules {
                cmd!(sh, "modprobe {module}").run().map_err(|e| LuaError::external(e.to_string()))?
            }
            Ok(())
        });

        methods.add_method_mut("mount", |_, this,
                (fstype, options, source, target, data): (String, String, String, String, String)| {
            println!("[KERNEL]: mount -t {} -o {} {} {}", fstype, options, source, target);
            if !Path::new(&target).exists() {
                std::fs::create_dir_all(&target).map_err(|e| LuaError::external(e.to_string()))?;
            }
            this.mountpoints.push(Mount::builder()
                .fstype(fstype.as_str())
                .flags(parse_mountflags(&options))
                .data(&data)
                .mount(source, target)?);
            Ok(())
        });
    }

    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("probe_modules", |_, this| {
            /* FIXME: this is probably not very efficient */
            Ok(this.probe_modules.clone().iter().map(|x| x.to_string()).collect::<Vec<_>>())
        });

        fields.add_field_method_set("probe_modules", |_, this, modules: Vec<String>| {
            this.probe_modules.extend(modules);
            Ok(())
        });
    }
}

fn parse_mountflags(opts: &str) -> MountFlags {
    let mut flags = MountFlags::empty();
    for opt in opts.split(',') {
        match opt {
            "bind" => flags |= MountFlags::BIND,
            "dirsync" => flags |= MountFlags::DIRSYNC,
            "mandlock" => flags |= MountFlags::MANDLOCK,
            "move" => flags |= MountFlags::MOVE,
            "noatime" => flags |= MountFlags::NOATIME,
            "nodev" => flags |= MountFlags::NODEV,
            "nodiratime" => flags |= MountFlags::NODIRATIME,
            "noexec" => flags |= MountFlags::NOEXEC,
            "nosuid" => flags |= MountFlags::NOSUID,
            "rdonly" => flags |= MountFlags::RDONLY,
            "rec" => flags |= MountFlags::REC,
            "relatime" => flags |= MountFlags::RELATIME,
            "remount" => flags |= MountFlags::REMOUNT,
            "silent" => flags |= MountFlags::SILENT,
            "strictatime" => flags |= MountFlags::STRICTATIME,
            "synchronous" => flags |= MountFlags::SYNCHRONOUS,
            _ => println!("[KERNEL]: unknown mount option: {}", opt),
        }
    }
    flags
}
