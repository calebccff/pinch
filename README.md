# pinch

A fast initramfs PID-1 written in Rust and driven by Lua.

## Building

Small release builds:

```sh
RUSTFLAGS="-C target-feature=+crt-static -Zlocation-detail=none -C strip=symbols" \
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
