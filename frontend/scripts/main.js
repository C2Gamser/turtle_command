console.log("Hellar, wog.");

class Coordinates {
    constructor(x, y, z) {
        this.x = x;
        this.y = y;
        this.z = z;
    }

    toString() {
        return `X: ${this.x} Y: ${this.y} Z: ${this.z}`
    }
}

class Slot {
    constructor(name, count) {
        this.name = name;
        this.count = count;
    }

    toString() {
        if (this.count > 0) {
            return `${this.name} x${this.count}`
        } else {
            return "Empty"
        }
    }
}

class Inventory {
    constructor(size, slots) {
        this.size = size;
        this.slots = slots;
    }
}

class Turtle {
    constructor(connected, coords, equipped_left, equipped_right, fuel, id, inventory) {
        this.connected = connected;
        this.coordinates = new Coordinates(coords.x, coords.y, coords.z);

        if (equipped_left) {
            this.equipped_left = new Slot(equipped_left.name, equipped_left.count);
        } else {
            this.equipped_left = new Slot("", 0);
        }

        if (equipped_right) {
            this.equipped_right = new Slot(equipped_right.name, equipped_right.count);
        } else {
            this.equipped_right = new Slot("", 0);
        }

        this.fuel = fuel;
        this.id = id;
        this.inventory = inventory;
    }

      // Getter
    get div() {
        let turtle_div = document.createElement("div");
        turtle_div.id = "turtleDiv"+this.id;

        // ID
        let elem = document.createElement("div");
        elem.id = "id";
        elem.innerText = "Turtle ID: "+this.id;
        turtle_div.appendChild(elem);

        // Coordinates
        elem = document.createElement("div");
        elem.id = "coordinates";
        elem.innerText = this.coordinates.toString();
        turtle_div.appendChild(elem);

        // Equipped left
        elem = document.createElement("div");
        elem.id = "equipped_left";
        elem.innerText = "Left Slot: "+this.equipped_left.toString();
        turtle_div.appendChild(elem);

        // Equipped right
        elem = document.createElement("div");
        elem.id = "equipped_right";
        elem.innerText = "Right Slot: "+this.equipped_right.toString();
        turtle_div.appendChild(elem);

        // Fuel
        elem = document.createElement("div");
        elem.id = "fuel";
        elem.innerText = "Fuel: "+this.fuel;
        turtle_div.appendChild(elem);

        return turtle_div
    }
}

// Standin for temporary debug pourpuses
fetch("/turtles/0.json")
  .then((response) => response.json())
  .then((data) => {
    let r = data;
    console.log(r)
    let this_turtle = new Turtle(r.connected, r.coordinates, r.equipped_left, r.equipped_right, r.fuel, r.id, r.inventory);
    document.getElementById("turtleContainer").appendChild(this_turtle.div);
  });