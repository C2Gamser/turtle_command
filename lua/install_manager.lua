-- For installation of turtle_command run:
-- wget run https://raw.githubusercontent.com/C2Gamser/turtle_command/refs/heads/master/lua/install_manager.lua true

local download_list = {
    "turtle_command.lua",
    "install_manager.lua",
    "move_utilities.lua",
    "thready.lua",
    "logging.lua"
}

local first_install = arg[1]

-- Helper function to return the server url
local function fetch_url()
    local url_file = fs.open("turtle_command/url.txt","r")
    local url = url_file.readLine()
    url_file.close()
    return url
end

local url = nil

if not first_install then
    url = fetch_url()
end

for i, v in pairs(download_list) do
    local response, fail_reason = nil, nil
    if first_install then
        response, fail_reason = http.get("https://raw.githubusercontent.com/C2Gamser/turtle_command/refs/heads/master/lua/"..v)
    else
        response, fail_reason = http.get(url.."/lua/"..v)
    end

    if fail_reason then
        print(fail_reason..". Getting "..v.." failed.")
    else
        local file = fs.open("turtle_command/"..v, "w")
        file.write(response.readAll())
        response.close()
        print("Got "..v..".")
    end
end

-- Removes the install manager if it is in the wrong place, e.g. on first installation
if fs.exists("install_manager.lua") and fs.exists("turtle_command/install_manager.lua") then
    fs.delete("install_manager.lua")
end

if first_install then
    local file = fs.open("startup.lua", "w")
    file.write("shell.run('install_manager.lua')")
    file.close()
end


shell.run("turtle_command/turtle_command.lua")