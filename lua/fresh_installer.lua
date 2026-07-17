-- For installation of turtle_command run:
-- wget run https://raw.githubusercontent.com/C2Gamser/turtle_command/refs/heads/master/lua/fresh_installer.lua

local download_list = {
    "turtle_command.lua",
    "install_manager.lua",
    "move_utilities.lua",
    "thready.lua",
    "logging.lua"
}

for i, v in pairs(download_list) do
    local response, fail_reason = nil, nil
    response, fail_reason = http.get("https://raw.githubusercontent.com/C2Gamser/turtle_command/refs/heads/master/lua/"..v)

    if fail_reason then
        print(fail_reason..". Getting "..v.." failed.")
    else
        local file = fs.open("turtle_command/"..v, "w")
        file.write(response.readAll())
        response.close()
        print("Got "..v..".")
    end
end

local file = fs.open("startup.lua", "w")
file.write("shell.run('install_manager.lua')")
file.close()

shell.run("turtle_command/install_manager.lua")