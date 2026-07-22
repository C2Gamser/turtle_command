local mv = require("utilities")
local sha = require("sha1")
local thready = require("thready")

-- Helper function to return url, api_key
local function fetch_conneciton_data()
    local url = settings.get("url")
    local api_key = settings.get("api_key")

    if url == "" then
        url = nil
    end

    if api_key == "" then
        api_key = nil
    end

    return url, api_key
end

-- Makes sure that all the files that must exist, do
-- Also sets up a settings file for the api key, ping time, and server url
-- In the case of the direction file, it will not allow the user to continue the program unless it has a direciton in it (n, s, e, w)
local function setup_files()
    if not settings.load("turtle_command/config.settings") then
        settings.set("api_key", "")

        settings.define("api_key", {
            description = "The API key required to establish a websocket connection with the server. On the server side this is located in api_key.txt",
            default = "",
            type = "string",
        })

        settings.set("url", "")

        settings.define("url", {
            description = "The URL of the server that this turtle will connect to.",
            default = "",
            type = "string",
        })

        settings.save("turtle_command/config.settings")
    end

    if not fs.exists("turtle_command/facing.txt") then
        local file = fs.open("turtle_command/facing.txt","w")
        file.close()
    end

    if not fs.exists("turtle_command/block_cache.txt") then
        local file = fs.open("turtle_command/block_cache.txt","w")
        file.close()
    end

    local url, api_key = fetch_conneciton_data()
    if not url then
        mv.single_color_print("Warning: No URL in turtle_command/confing.settings!", colors.yellow)
    end

    if not api_key then
        mv.single_color_print("Warning: No API key in turtle_command/confing.settings!", colors.yellow)
    end

    -- Errors if there is no direction in the direction file as the turtle NEEDS to know its direction.
    if mv.read_first_line("turtle_command/facing.txt") == nil then
        mv.single_color_print("Error: No direction key in turtle_command/facing.txt, you must manually insert the direction this turtle is facing!", colors.red)
        error()
    end
end

-- Returns instruction, data
-- Kind is always a string representing how to deal with response
local function parse_response(input)
    local decoded_json = textutils.unserialiseJSON(input)
    return decoded_json.instruction, decoded_json.data
end

-- Gets a bunch of data about this turtle
local function fetch_own_status()
    local x, y, z = nil, nil, nil

    local counter = 1
    while not x and counter < 5 do
        x, y, z = gps.locate(2)
        counter = counter + 1
    end

    local computer_id = os.getComputerID()
    local equipped_left = turtle.getEquippedLeft()
    local equipped_right = turtle.getEquippedRight()
    local inventory = mv.scan_own_inventory()

    local fuel = turtle.getFuelLevel()

    -- we set connected to true here as if this message gets sent, then we must be connected
    if textutils.serialiseJSON(inventory) == "{}" then
        inventory = nil
    end
    local my_data = {
        id = computer_id,
        connected = true,
        equipped_left = equipped_left,
        equipped_right = equipped_right,
        coordinates = {x = x, y = y, z = z},
        inventory_contents = inventory,
        inventory_size = 16,
        fuel = fuel, }

    return my_data
end

-- Utility function to change a direction as a letter to a coordinate number. E.g. turns "n" into a Z offset of -1
-- Returns offset_x, offset_z
local function facing_offset()
    local facing = mv.read_first_line("turtle_command/facing.txt")
    local offset_x = 0
    local offset_z = 0
    if facing == "n" then
        offset_z = -1
    elseif facing == "s" then
        offset_z = 1
    else
        offset_z = 0
    end
    if facing == "e" then
        offset_x = 1
    elseif facing == "w" then
        offset_x = -1
    else
        offset_x = 0
    end
    return offset_x, offset_z
end

-- Adds a block to the block cache
local function append_block_cache(block_cache)
    local f = fs.open("turtle_command/block_cache.txt", "r")
    local block_cash_old = textutils.unserialise(f.readAll())
    f.close()

    -- If there werent any blocks in the old cache, block_cash_old = nil, causing an error, thus this is here.
    if not block_cash_old then
        block_cash_old = {}
    end

    -- This is the structure of an item in block cache: {name = data.name, x = x, y = y, z = z}
    -- data.name is the name of a block
    for i, v in pairs(block_cache) do
        -- Only append non-turtle blocks as other turtles should be tracked by the server already
        if v.name ~= "computercraft:turtle_advanced" or v.name ~= "computercraft:turtle_normal" then
            block_cash_old[#block_cash_old+1] = v
        end
    end

    f = fs.open("turtle_command/block_cache.txt", "w")
    f.write(textutils.serialise(block_cash_old))
    f.close()
end

-- Uses turtle.inspect to inspect all 6 blocks around it, appending them to the block cache.
local function append_inspect_all()
    local x, y, z = gps.locate()

    local offset_x, offset_z = facing_offset()

    local block_cache = {}
    local has_block, data = turtle.inspectUp()
    if has_block then block_cache[#block_cache+1] = {{name = data.name, states = data.state}, {x = x, y = y+1, z = z}}
    else block_cache[#block_cache+1] = {{name = "minecraft:air", states = {}}, {x = x, y = y+1, z = z}} end

    local has_block, data = turtle.inspectDown()
    if has_block then block_cache[#block_cache+1] = {{name = data.name, states = data.state}, {x = x, y = y-1, z = z}}
    else block_cache[#block_cache+1] = {{name = "minecraft:air", states = {}}, {x = x, y = y-1, z = z}} end

    local has_block, data = turtle.inspect()
    if has_block then block_cache[#block_cache+1] = {{name = data.name, states = data.state}, {x = x + offset_x, y = y, z = z + offset_z}}
    else block_cache[#block_cache+1] = {{name = "minecraft:air", states = {}}, {x = x + offset_x, y = y, z = z + offset_z}} end

    mv.left()

    local offset_x, offset_z = facing_offset()

    local has_block, data = turtle.inspect()
    if has_block then block_cache[#block_cache+1] = {{name = data.name, states = data.state}, {x = x + offset_x, y = y, z = z + offset_z}}
    else block_cache[#block_cache+1] = {{name = "minecraft:air", states = {}}, {x = x + offset_x, y = y, z = z + offset_z}} end

    mv.left()

    local offset_x, offset_z = facing_offset()

    local has_block, data = turtle.inspect()
    if has_block then block_cache[#block_cache+1] = {{name = data.name, states = data.state}, {x = x + offset_x, y = y, z = z + offset_z}}
    else block_cache[#block_cache+1] = {{name = "minecraft:air", states = {}}, {x = x + offset_x, y = y, z = z + offset_z}} end

    mv.left()

    local offset_x, offset_z = facing_offset()

    local has_block, data = turtle.inspect()
    if has_block then block_cache[#block_cache+1] = {{name = data.name, states = data.state}, {x = x + offset_x, y = y, z = z + offset_z}}
    else block_cache[#block_cache+1] = {{name = "minecraft:air", states = {}}, {x = x + offset_x, y = y, z = z + offset_z}} end

    mv.left()

    append_block_cache(block_cache)
end

-- Utility function, used such that each time the turtle moves it will cache the blocks directly above and below it.
-- Better than running append_inspect_all() each time as that slows the turtle down massively.
local function cache_updown_move()
    local x, y, z = gps.locate()

    local block_cache = {}
    local has_block, data = turtle.inspectUp()
    if has_block then block_cache[#block_cache+1] = {{name = data.name, states = data.state}, {x = x, y = y+1, z = z}}
    else block_cache[#block_cache+1] = {{name = "minecraft:air", states = {}}, {x = x, y = y+1, z = z}} end

    local has_block, data = turtle.inspectDown()
    if has_block then block_cache[#block_cache+1] = {{name = data.name, states = data.state}, {x = x, y = y-1, z = z}}
    else block_cache[#block_cache+1] = {{name = "minecraft:air", states = {}}, {x = x, y = y-1, z = z}} end

    append_block_cache(block_cache)

    return true
end

-- Formats a message like {instruction = instruction, data = data} and then json serializes it
local function format_message(instruction, data)
    local message = {instruction = instruction, data = data}
    return textutils.serialiseJSON(message)
end

-- Hashes the file and sends the hash to the server, the server will likely send back fileIdentical
-- or it may send back a fileData command which downloads a file
local function verify_file_with_server(websocket, file_name)
    if fs.exists("turtle_command/"..file_name) then
        local file = fs.open("turtle_command/"..file_name, "r")
        local contents = file.readAll()
        file.close()
        local hash = sha.sha1(contents)
        local send_data = {file_name, hash}
        websocket.send(format_message("verifyFile", textutils.serialiseJSON(send_data)))
    end
end

-- Runs verify_file_with_server on every lua file in turtle_command/
local function verify_lua_files(websocket)
    local file_list = fs.list("turtle_command/")
    for i, v in pairs(file_list) do
        local i, j = string.find(v, "[^%.]*%.lua")
        if i ~= nil then
            verify_file_with_server(websocket, v)
        end
    end
end

-- Opens the block cache file, sends all of the data to the server, then clears the file.
local function send_block_cache(websocket)
    local f = fs.open("turtle_command/block_cache.txt", "r")
    local block_cache = f.readAll()
    f.close()
    if not block_cache then
        return
    end

    block_cache = textutils.unserialise(block_cache)

    f = fs.open("turtle_command/block_cache.txt","w")
    f.close()

    if block_cache then
        local message = format_message("sendBlocks", textutils.serialiseJSON(block_cache))
        websocket.send(message)
    end
end

-- Sends a websocket message with all the turtle's data
local function ws_register(websocket)
    local send_data = fetch_own_status()

    -- Sets up the inventory to be correctly formatted
    send_data["inventory"] = {send_data.inventory_size, send_data.inventory_contents}
    send_data.inventory_contents = nil
    send_data.inventory_size = nil

    if send_data.inventory[2] == nil then
        send_data.inventory[2] = {}
    end

    -- Makes sure that empty inventory slots are still counted as null when serialized to json
    for i = 1, 16 do
        if send_data.inventory[2][i] == nil then
            send_data.inventory[2][i] = textutils.json_null
        end
    end

    local message = format_message("register", textutils.serialiseJSON(send_data))
    websocket.send(message)
end

-- Sends a websocket message to acknowledge a received message
local function ws_acknowledge(websocket)
    local message = format_message("acknowledge", textutils.serialiseJSON(textutils.json_null))
    websocket.send(message)
end

local function ws_save_file(data)
    local file_data = textutils.unserialiseJSON(data)
    local file_name = file_data["file_name"]
    local file_content = file_data["content"]

    local file = fs.open("turtle_command/"..file_name, "w")
    file.write(file_content)
    file.close()

    mv.single_color_print("SF: Wrote to "..file_name, colors.gray)

    if file_name == "turtle_command.lua" then
        -- The sleep below is CRITICAL for the program to work properly
        -- Likely due to how thready works, or due to a bug in computercraft itself,
        -- when verify_lua_files runs and detects that turtle_command.lua needs to be replaced,
        -- it starts a new copy of it too quickly, so when the old program ends,
        -- the websocket the new program uses is then shutdown by the old program.
        -- This order somehow prevents that from happening
         -- Start a new instance
        os.sleep(0.05)
        shell.openTab("turtle_command/turtle_command.lua")
        -- Shutdown this instance of turtle_command
        os.queueEvent("terminate")

    end
end

-- Creates a websocket with the server address in url.txt
local function establish_websocket()
    local url, api_key = fetch_conneciton_data()

    if url == nil then
        error("There is no URL in turtle_command/config.settings")
    end

    -- The sub here gets rid of the "https" so that it can be replaced with "ws"
    -- Note: We also submit the ID so the rust server can track which websocket is which
    local server_address = "ws"..url:sub(5, -1).."/websocket?id="..os.getComputerID()
    mv.single_color_print("Establishing websocket connection to "..server_address, colors.gray)
    local socket, fail_reason = http.websocket({url = server_address, timeout = 5, headers = {api_key = api_key}})

    if not socket then
        print(fail_reason)
    else
        mv.single_color_print("Websocket connected!", colors.gray)
    end

    return socket
end

-- Handles movement instructions
local function handle_move(data)
    if data == "turnLeft" or data == "left" or data == "l" then
        return mv.left()
    elseif data == "turnRight" or data == "right" or data == "r" then
        return mv.right()
    elseif data == "forward" or data == "f" then
        return turtle.forward()
    elseif data == "up" or data == "u" then
        return turtle.up()
    elseif data == "down" or data == "d" then
        return turtle.down()
    end
end

-- Handles run length encoding paths
-- Data should be formatted as such:
-- (letter)(number) etc...
-- for example, l4r5u12d1rl means left 4, right 5, up 12, down 1, right, left
local function handle_path(websocket, data)
    local raw_list={}
    data:gsub("%a%d*",function(c) table.insert(raw_list, c) end)

    for i, v in pairs(raw_list) do
        local action = string.sub(v, 1, 1)
        local count = tonumber(string.sub(v, 2, -1))

        -- Handles the single letter commands
        if count == nil then
            count = 1
        end

        print(v)

        -- Handles errors along the way
        for c = 1, count do
            local success, reason = handle_move(action)
            print(reason)
            if reason == "Movement obstructed" then
                websocket.send(format_message("movementObstructed", ""))
            end

            if not success then
                error(reason)
            end
        end
    end
end

-- Handles the terminate event so it shuts down the websocket before terminating
local function handle_terminate(websocket)
    websocket.close()

    -- Shuts down thready quickly
    thready.running = false
    thready.websocket = nil
    websocket = nil

    -- Shutdown this multishell tab if it isn't the only one
    if multishell.getCount() > 1 then
        os.queueEvent("terminate")
    else
        mv.single_color_print("Terminated", colors.red)
    end
end

local function handle_websocket_message(websocket, event_name, url, message, is_binary)
    if not mv.verify_address(url) then
        error("Recieved message from non-target web address!")
    elseif is_binary then
        error("Recieved binary response from websocket!")
    end

    local kind, data = parse_response(message)

    if kind == "move" then
        handle_move(data)
    elseif kind == "movementPath" then
        handle_path(websocket, data)
    elseif kind == "register" then
        ws_register(websocket)
    elseif kind == "fileData" then
        ws_save_file(data)
    elseif kind == "fileIdentical" then
        -- Do nothing
    elseif  kind == "status" and data == "successful" then
        -- Do nothing
    elseif kind == "testBlockSend" then -- DEBUG
        append_inspect_all()
        send_block_cache(websocket)
    else
        if not data then
            data = ""
        end
        mv.single_color_print("Rep K:"..kind.." D:"..data.." unknown.", colors.lightGray)
    end

    -- TODO: Deal with more responses
end

local function persistent_connect(websocket)
    local counter = 0
    while true do
        if io.type(websocket) == "file" then
            websocket.close()
        end

        websocket = establish_websocket()
        if not websocket then
            write("Retrying.")
            os.sleep(1)
            write(".")
            os.sleep(1)
            write(".")
            os.sleep(1)
            print("")
            counter = counter + 1
        else
            thready.websocket = websocket
            print("Took "..counter.." attempts to connect.")
            ws_register(websocket)
            return websocket
        end
    end
end

local function handle_websocket_closure(websocket)
    print("Websocket unexpectedly closed!")
    print("Attempting reconnect.")

    local websocket = persistent_connect(websocket)
    thready.spawn("verify_lua_files", verify_lua_files, websocket)
end

term.clear()
term.setCursorPos(1,1)
mv.single_color_print("Starting turtle command!", colors.green)

setup_files()

local websocket = establish_websocket()
if not websocket then
    websocket = persistent_connect(websocket)
end

thready.websocket = websocket

-- Runs verify_file_with_server on every lua file in turtle_command/
-- IMPORTANT: If the turtle sends an updated version of turtle_command.lua,
-- the program will start a NEW copy of itself with the new file and close the old process
thready.spawn("verify_lua_files", verify_lua_files, websocket)

-- NOTE: Change made by C2, the first argument passed to all listeners is the global websocket!
thready.listen("websocket_handler", "websocket_message", handle_websocket_message)
thready.listen("websocket_closed_handler", "websocket_closed", handle_websocket_closure)
thready.listen("terminate_handler", "terminate", handle_terminate)
thready.kill_set_on_error = false
thready.stop_on_error = false
thready.main_loop()