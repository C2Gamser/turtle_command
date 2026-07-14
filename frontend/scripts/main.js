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
    constructor(size, slot_list) {
        this.size = size;
        this.slots = [];
        for (let i of slot_list) {
            if (i == null) {
                this.slots.push(new Slot("Empty", 0))
            } else {
                this.slots.push(new Slot(i.name, i.count))
            }

        }
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
        this.inventory = new Inventory(inventory.size, inventory.slots);
    }

      // Getter
    get div() {
        let turtle_div = document.createElement("div");
        turtle_div.id = this.id;
        turtle_div.classList.add("turtle")

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

        // Inventory
        // elem = document.createElement("div");
        // elem.id = "inventory";
        // elem.class = "inventoryContainer";
        // elem.innerText = "Inventory: ";
        // turtle_div.appendChild(elem);
        // // Inventory fulfillment center
        // for (let i = 0; i < this.inventory.size; i ++) {
        //     let elem2 = document.createElement("div");

        //     if (this.inventory.slots[i] == null) {
        //         elem2.innerText = "Empty"
        //     } else {
        //         elem2.innerText = this.inventory.slots[i].toString()
        //     }

        //     elem.appendChild(elem2)
        // }


        turtle_div.appendChild(elem)

        return turtle_div
    }
}

// Gets a list of connected turtle ids
fetch("/connected_ids")
  .then((response) => response.json())
  .then((data) => {
    select = document.getElementById("turtleSelector");

    for (var i = 0; i < data.length; i++) {
      var option = document.createElement("option");
      option.value = data[i];
      option.textContent = data[i];
      select.appendChild(option);
    };
  });

// Standin for temporary debug pourpuses
fetch("/turtles/0.json")
  .then((response) => response.json())
  .then((data) => {
    let r = data;
    console.log(r)
    // Sets up the turtle box which contains a single turtle and the power button
    let turtle_flex = document.createElement("div");
    turtle_flex.classList.add("turtleBoxGrid")

    // Creates a button inside the turtle box
    let turtle_button = document.createElement("button");
    turtle_button.id = "turtlePowerButton";
    turtle_button.classList.add("powerButton")
    // turtle_button.textContent = "P";
    turtle_flex.appendChild(turtle_button);

    // Creates a turtle inside the turtle box
    let this_turtle = new Turtle(r.connected, r.coordinates, r.equipped_left, r.equipped_right, r.fuel, r.id, r.inventory);
    turtle_flex.appendChild(this_turtle.div)

    document.getElementById("turtleContainer").appendChild(turtle_flex);
    document.getElementById("turtleContainer").classList.add("turtleContainerGrid")
  });