--!strict

---@class Udev
---@field match_subsystem fun(subsystem: string): nil
---@field scan fun(): Device[]

---@class Device
---@field syspath string
---@field devpath string

print(string.format("Pinch starting up on %s (%s)", deviceinfo["name"], os.date()))
print(string.format("Framebuffer disabled: %s", deviceinfo["no_framebuffer"]))
print(string.format("Kernel modules: %s", table.concat(kernel.probe_modules, ", ")))

local splash = require("splash")

local function mount_special()
    -- fstype, options, source, target, data
    kernel:mount("proc", "nodev,noexec,nosuid", "proc", "/proc", "")
    kernel:mount("sysfs", "nodev,noexec,nosuid", "sys", "/sys", "")
    kernel:mount("devtmpfs", "noexec,nosuid", "dev", "/dev", "mode=0755")
    kernel:mount("tmpfs", "nosuid,nodev", "tmpfs", "/run", "mode=0755")
end

--[[ Mount configs, debugfs, devpts for debugging/telnet ]]--
local function mount_debug()
    kernel:mount("configfs", "nodev,noexec,nosuid", "configfs", "/sys/kernel/config", "")
    kernel:mount("debugfs", "nodev,noexec,nosuid", "debugfs", "/d", "")
    kernel:mount("devpts", "", "devpts", "/dev/pts", "")
end

local function system_setup()
    -- Mount /proc, /sys, and /dev, etc...
    mount_special()
    -- Probe kernel modules
    kernel:probe()
    -- Then start the udev daemon
    run("/sbin/udevd -dD --resolve-names=never")
    -- Configure firmware search path
    write("/sys/module/firmware_class/parameters/path", "/lib/firmware/postmarketos")
end

local function udev_scan()
    local udev = Udev()

    -- We only care about block, input, and graphics devices
    udev:match_subsystem("block")
    udev:match_subsystem("input")
    udev:match_subsystem("graphics")

    return udev:scan()
end

if not sandbox then
    system_setup()
end

local devices = udev_scan()

for i,device in ipairs(devices) do
    print(("found device: %s"):format(device))
end
