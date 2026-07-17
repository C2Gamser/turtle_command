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

return {
    left = turnLeft,
    right = turnRight,
    read_first_line = read_first_line,
    rewrite_file = rewrite_file,
    verify_address = verify_address,
    scan_own_inventory = scan_own_inventory
}