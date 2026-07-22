local sha = require("sha1")

-- Opens, reads, and returns the first line of a file.
local function read_first_line(filepath)
    local f = fs.open(filepath, "r")
    local data = f.readLine()
    f.close()

    return data
end

-- Opens and replaces a file with a string.
local function rewrite_file(filepath, string)
    local f = fs.open(filepath, "w")
    f.write(string)
    f.close()
end

local facing_filepath = "turtle_command/facing.txt"

-- Updates the direction file when turning left
local function turnLeft()
    local dir = read_first_line(facing_filepath)
    local rewrite_with = ""
    if dir == "n" then
        rewrite_with = "w"
    elseif dir == "w" then
        rewrite_with = "s"
    elseif dir == "s" then
        rewrite_with = "e"
    elseif dir == "e" then
        rewrite_with = "n"
    end
    turtle.turnLeft()
    rewrite_file(facing_filepath, rewrite_with)
    return true
end

-- Updates the direction file when turning right
local function turnRight()
    local dir = read_first_line(facing_filepath)
    local rewrite_with = ""
    if dir == "n" then
        rewrite_with = "e"
    elseif dir == "e" then
        rewrite_with = "s"
    elseif dir == "s" then
        rewrite_with = "w"
    elseif dir == "w" then
        rewrite_with = "n"
    end

    turtle.turnRight()
    rewrite_file(facing_filepath, rewrite_with)
    return true
end

-- Takes in a URL and compares it to our target URL
-- If they match, return true, else return false
-- Parses the input data so that http://127.0.0.1:8000 is reduced to 127.0.0.1:8000
-- Also throws away the rest of the url, e.g. http://127.0.0.1:8000/command?id=5 turns into 127.0.0.1:8000 also
local function verify_address(url)
    local i, j = string.find(url, "//[^/]+/?")
    url = string.gsub(string.sub(url, i, j), "/", "")

    settings.load("turtle_command/config.settings")
    local my_url = settings.get("url")
    i, j = string.find(my_url, "//[^/]+/?")
    my_url = string.gsub(string.sub(my_url, i, j), "/", "")

    return url == my_url
end

-- Returns a table containing all the items in its inventory
local function scan_own_inventory()
    local inventory = {}
    for i = 1, 16 do
        inventory[i] = turtle.getItemDetail(i)
    end

    return inventory
end

local function try_set_color(color)
    if term.isColor() then
        term.setTextColor(color)
    end
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

-- Takes in a string, sets the color to the input color, prints the string, then sets color to white
local function single_color_print(string, color)
    try_set_color(color)
    print(string)
    try_set_color(colors.white)
end

-- Determines wether or not the input string is in the input table
local function is_in(string, table)
    for i, v in pairs(table) do
        if v == string then
            return true
        end
    end
    return false
end

-- Utility function to change a direction as a letter to a coordinate number. E.g. turns "n" into a Z offset of -1
-- Returns offset_x, offset_z
local function facing_offset()
    local facing = read_first_line("turtle_command/facing.txt")
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

    turnLeft()

    local offset_x, offset_z = facing_offset()

    local has_block, data = turtle.inspect()
    if has_block then block_cache[#block_cache+1] = {{name = data.name, states = data.state}, {x = x + offset_x, y = y, z = z + offset_z}}
    else block_cache[#block_cache+1] = {{name = "minecraft:air", states = {}}, {x = x + offset_x, y = y, z = z + offset_z}} end

    turnLeft()

    local offset_x, offset_z = facing_offset()

    local has_block, data = turtle.inspect()
    if has_block then block_cache[#block_cache+1] = {{name = data.name, states = data.state}, {x = x + offset_x, y = y, z = z + offset_z}}
    else block_cache[#block_cache+1] = {{name = "minecraft:air", states = {}}, {x = x + offset_x, y = y, z = z + offset_z}} end

    turnLeft()

    local offset_x, offset_z = facing_offset()

    local has_block, data = turtle.inspect()
    if has_block then block_cache[#block_cache+1] = {{name = data.name, states = data.state}, {x = x + offset_x, y = y, z = z + offset_z}}
    else block_cache[#block_cache+1] = {{name = "minecraft:air", states = {}}, {x = x + offset_x, y = y, z = z + offset_z}} end

    turnLeft()

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


return {
    left = turnLeft,
    right = turnRight,
    read_first_line = read_first_line,
    rewrite_file = rewrite_file,
    verify_address = verify_address,
    scan_own_inventory = scan_own_inventory,
    try_set_color = try_set_color,
    single_color_print = single_color_print,
    is_in = is_in,
    append_inspect_all = append_inspect_all,
    cache_updown_move = cache_updown_move,
    
}