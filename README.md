# pinch

A fast initramfs PID-1 written in Rust and driven by Lua.

```
[    0.440249] Run /init as init process
[Starting Pinch]
-- Initializing Lua environment
-- Running script: init.env.lua
-- Running script: /usr/share/pinch/device-qemu-amd64-init.lua
-- Running script: init.lua
[LUA] Pinch starting up on QEMU amd64 (Sun Mar 17 02:23:56 2024)
[LUA] Framebuffer disabled: false
[LUA] Kernel modules: 
[RUST]: probing modules: {"libcomposite", "vfat", "virtio-gpu", "virtio_input", "virtio_pci", "virtio_blk", "evdev"}
$ modprobe libcomposite
$ modprobe vfat
$ modprobe virtio-gpu
[    0.478948] ACPI: bus type drm_connector registered
$ modprobe virtio_input
$ modprobe virtio_pci
[    0.506230] ACPI: \_SB_.LNKB: Enabled at IRQ 10
[    0.508777] input: QEMU Virtio Mouse as /devices/pci0000:00/0000:00:02.0/virtio0/input/input2
[    0.519092] ACPI: \_SB_.LNKC: Enabled at IRQ 11
[    0.521646] input: QEMU Virtio Keyboard as /devices/pci0000:00/0000:00:03.0/virtio1/input/input3
[    0.530380] ACPI: \_SB_.LNKD: Enabled at IRQ 11
[    0.540925] ACPI: \_SB_.LNKA: Enabled at IRQ 10
[    0.546178] [drm] pci: virtio-vga detected at 0000:00:05.0
[    0.547478] Console: switching to colour dummy device 80x25
[    0.548359] virtio-pci 0000:00:05.0: vgaarb: deactivate vga console
[    0.550564] [drm] features: +virgl +edid -resource_blob -host_visible
[    0.550565] [drm] features: -context_init
[    0.553883] [drm] number of scanouts: 1
[    0.554524] [drm] number of cap sets: 2
[    0.566724] [drm] cap set 0: id 1, max-version 1, max-size 308
[    0.568695] [drm] cap set 1: id 2, max-version 2, max-size 1384
[    0.570799] [drm] Initialized virtio_gpu 0.1.0 0 for 0000:00:05.0 on minor 0
[    0.572532] fbcon: virtio_gpudrmfb (fb0) is primary device
[    0.574948] Console: switching to colour frame buffer device 80x30
[    0.581496] virtio-pci 0000:00:05.0: [drm] fb0: virtio_gpudrmfb frame buffer device
$ modprobe virtio_blk
[    0.601218] virtio_blk virtio4: 32/0/0 default/read/poll queues
[    0.609238] virtio_blk virtio4: [vda] 8388608 512-byte logical blocks (4.29 GB/4.00 GiB)
[    0.615545] GPT:Primary header thinks Alt. header is not at the end of the disk.
[    0.617434] GPT:3692543 != 8388607
[    0.618935] GPT:Alternate GPT header not at the end of the disk.
[    0.621391] GPT:3692543 != 8388607
[    0.622624] GPT: Use GNU Parted to correct GPT errors.
[    0.624082]  vda: vda1 vda2
$ modprobe evdev
$ /sbin/udevd -d --resolve-names=never
[    0.629269] udevd[805]: starting version 3.2.14
[    0.631439] udevd[806]: starting eudev-3.2.14
-- /sys/module/firmware_class/parameters/path > /lib/firmware/postmarketos
[RUST]: udev: matching subsystem: block
[RUST]: udev: matching subsystem: input
[RUST]: udev: matching subsystem: graphics
[RUST]: udev: scanning devices
[LUA] found device: /dev/input/event1
[LUA] found device: /dev/input/event2
[LUA] found device: /dev/fb0
[LUA] found device: /dev/vda
[LUA] found device: /dev/vda1
[LUA] found device: /dev/vda2
[LUA] found device: /dev/input/event0
```

## Building

Small release builds:

```sh
RUSTFLAGS="-Zlocation-detail=none -C strip=symbols" \
    cargo +nightly build --release \
    -Z build-std=std,panic_abort \
    -Z build-std-features=panic_immediate_abort \
    --target x86_64-unknown-linux-gnu
```

## Size analysis

With following modules: `756k`

```toml
anyhow = { version = "1.0.81", features = [] }
mlua = { version = "0.9.6", features = ["luau", "vendored"] }
```

Removing `anyhow` brings us down to `716k` (saves `40k`), so not a worthwhile
tradeoff.

Switching from `luau` to `lua54` brings it down to `384k` (saves `382k`). Bah!

## pmOS ramdisk stats

Ignoring kernel modules which vary per kernel package/arch/device.

Initramfs for qemu-amd64 is `2.6M`

```sh
/tmp/extract # ls -lh lib/
total 1M
-rwxr-xr-x    1 root     root      634.6K Mar 14 22:02 ld-musl-x86_64.so.1
lrwxrwxrwx    1 root     root          17 Mar 14 22:02 libblkid.so.1 -> libblkid.so.1.1.0
-rwxr-xr-x    1 root     root      186.8K Mar 14 22:02 libblkid.so.1.1.0
lrwxrwxrwx    1 root     root          19 Mar 14 22:02 libc.musl-x86_64.so.1 -> ld-musl-x86_64.so.1
-r-xr-xr-x    1 root     root      287.4K Mar 14 22:02 libdevmapper.so.1.02
/tmp/extract # ls -lah bin
total 904K
drwxr-xr-x    2 root     root        4.0K Mar 14 22:02 .
drwxr-xr-x   14 root     root        4.0K Mar 14 22:02 ..
-rwxr-xr-x    1 root     root      789.8K Mar 14 22:02 busybox
-rwxr-xr-x    1 root     root      101.7K Mar 14 22:02 busybox-extras
lrwxrwxrwx    1 root     root          12 Mar 14 22:02 sh -> /bin/busybox
/tmp/extract # ls -lh usr/bin/
total 116K
-rwxr-xr-x    1 root     root       98.2K Mar 14 22:02 pbsplash
-rwxr-xr-x    1 root     root       13.9K Mar 14 22:02 unudhcpd
/tmp/extract # ls -lh usr/sbin
total 44K
-rwxr-xr-x    1 root     root       42.1K Mar 14 22:02 kpartx
/tmp/extract # ls -lh sbin
total 48K
-rwxr-xr-x    1 root     root       46.1K Mar 14 22:02 blkid
```

Replacing busybox would obviously be awesome, and maybe even feasible. We would
lose debug shell capabilities, but on devices that have the ability to choose
between ramdisks we could just have a debug variant.

Next biggest things are `libblkid` and `libdevmapper`. Given that `libblkid` was
just used to let `mdev` properly configure `/dev/disk/by-partlabel` we can
probably get rid of it.
