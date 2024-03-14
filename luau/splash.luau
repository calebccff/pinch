--!strict

print("splash.lua!")

local splash: Splash = {}

export type SplashConfig = {
    message: string,
    bottomText: string,
    animate: boolean,
}

function splash.start(config: SplashConfig)
    print("splash start!")
end

function splash.update(message: string, animate: boolean)
    print("splash update!")
end

function splash.stop()
    print("splash stop!")
end

-- Define the type Splash based on the attributes of the splash object which
-- contains our module functions.
export type Splash = typeof(splash)

return splash
