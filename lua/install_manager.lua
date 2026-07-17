-- For installation of turtle_command run:
-- wget run https://raw.githubusercontent.com/C2Gamser/turtle_command/refs/heads/master/lua/fresh_installer.lua

settings.load("turtle_command/config.settings")

local download_list = {
    "turtle_command.lua",
    "install_manager.lua",
    "utilities.lua",
    "thready.lua",
    "logging.lua",
    "sha1.lua"
}

local url = settings.get("url")

for i, v in pairs(download_list) do
    local response, fail_reason = nil, nil
    response, fail_reason = http.get(url.."/lua/"..v)

    if fail_reason then
        print(fail_reason..". Getting "..v.." failed.")
    else
        local file = fs.open("turtle_command/"..v, "w")
        file.write(response.readAll())
        response.close()
        print("Got "..v..".")
    end
end

shell.run("turtle_command/turtle_command.lua")