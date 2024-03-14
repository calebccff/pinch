--!strict

local splash= {}

function splash.start(config)
    print("splash start!")
end

function splash.update(message, animate)
    print("splash update!")
end

function splash.stop()
    print("splash stop!")
end

return splash
