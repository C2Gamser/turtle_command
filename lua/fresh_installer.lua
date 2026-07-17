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

if not fs.exists("turtle_command/url.txt") then
    local file = fs.open("turtle_command/url.txt","w")
    print("Input the target server url: ")
    local msg = read()
    file.write(msg)
    file.close()
end

if not fs.exists("turtle_command/api_key.txt") then
    local file = fs.open("turtle_command/api_key.txt","w")
    print("Input the api key: ")
    local msg = read()
    file.write(msg)
    file.close()
end

if not fs.exists("turtle_command/facing.txt") then
    local file = fs.open("turtle_command/facing.txt","w")
    print("Input the first letter of the direction the turtle is facing (e.g. w for west): ")
    local msg = read()
    file.write(msg)
    file.close()
end

if not fs.exists("turtle_command/block_cache.txt") then
    local file = fs.open("turtle_command/block_cache.txt","w")
    file.close()
end

if not fs.exists("turtle_command/keep_alive_time.txt") then
    local file = fs.open("turtle_command/keep_alive_time.txt","w")
    file.write("6")
    file.close()
end

shell.run("turtle_command/turtle_command.lua")