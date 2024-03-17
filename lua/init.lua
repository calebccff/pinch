--!strict

---@class write_sysfs fun(path: string, value: string): nil

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

if not sandbox then
    -- First probe kernel modules
    kernel:probe()
    -- Then start the udev daemon
    run("/sbin/udevd -d --resolve-names=never")
    write_sysfs("/sys/module/firmware_class/parameters/path", "/lib/firmware/postmarketos")
end

local udev = Udev()

-- We only care about block, input, and graphics devices
udev:match_subsystem("block")
udev:match_subsystem("input")
udev:match_subsystem("graphics")

for i,device in ipairs(udev:scan()) do
    print(("found device: %s"):format(device.devnode))
end
