use mlua::prelude::*;
use udev::{self, AsRawWithContext};

pub struct PinchUdev {
    udev: udev::Enumerator,
    mon: Option<udev::MonitorSocket>,
}

impl PinchUdev {
    pub fn new(udev: udev::Enumerator) -> Self {
        // FIXME: Have lua register callbacks for block/graphics/etc and then call into the monitor
        // event loop. Callbacks can choose to break out of the loop, e.g. when the root partition
        // has been found.
        PinchUdev { udev, mon: None }
    }
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
pub struct PinchDevice(udev::Device);

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
            // Ok(format!("{:24} {:16} {}",
            //     match this.0.devnode() { Some(x) => x.to_string_lossy().into_owned(), None => "".into()},
            //     match this.0.subsystem() { Some(x) => x.to_string_lossy().into_owned(), None => "".into()},
            //     this.0.syspath().to_string_lossy().into_owned()))
            Ok(format!("{:?}:\n{}", this.0.devnode(), this.0.attributes().map(|e| format!("\t{}: {}", e.name().to_string_lossy(), e.value().to_string_lossy())).collect::<Vec<_>>().join(", ") ))
        });
    }
}

