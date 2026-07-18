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
    constructor(r) {
        this.connected = r.connected;
        this.coordinates = new Coordinates(r.coordinates.x, r.coordinates.y, r.coordinates.z);

        if (r.equipped_left) {
            this.equipped_left = new Slot(r.equipped_left.name, r.equipped_left.count);
        } else {
            this.equipped_left = new Slot("", 0);
        }

        if (r.equipped_right) {
            this.equipped_right = new Slot(r.equipped_right.name, r.equipped_right.count);
        } else {
            this.equipped_right = new Slot("", 0);
        }

        this.fuel = r.fuel;
        this.id = r.id;
        this.inventory = new Inventory(r.inventory.size, r.inventory.slots);
    }
}

/**
 * Usage:
 * <x-turtle turtle_id=id live_update=bool></x-turtle>
 * where id is the turtle's id, e.g. 0 and live update is either true or false
 * live update can also be changed during runtime
 */
class TurtleComponent extends HTMLElement {
    connectedCallback() {
        // Sets up the turtle's elements
        this.innerHTML = `
            <button class="powerButton"></button>
            <div class="turtle_id"></div>
            <div class="turtle_coordinates"></div>
            <div class="turtle_fuel"></div>
        `;

        this.update();
        this.loop();
    }

    // Sets it up so the turtle auto fetches data every 1.5 seconds from the server
    loop() {
        setInterval(() => {
            if (this.getAttribute("live_update") == "true") {
                this.update();
            }
        }, 2000);
    };

    update() {
        // Fetches the turtle's data
        fetch("/turtles/"+this.getAttribute("turtle_id")+".json")
        .then((response) => response.json())
        .then((data) => {
            let r = data;

            let new_turt = new Turtle(r)
            let turtle_id_div = this.querySelector("div.turtle_id")
            turtle_id_div.innerText = `Turtle ID: ${new_turt.id}`

            let coordinates_div = this.querySelector("div.turtle_coordinates")
            coordinates_div.innerText = `${new_turt.coordinates.toString()}`

            let fuel_div = this.querySelector("div.turtle_fuel")
            fuel_div.innerText = `Fuel: ${new_turt.fuel.toString()}`

            this.setAttribute("data-connected", new_turt.connected)
        });
    }
}

export const registerTurtleComponent = () => {
    customElements.define('x-turtle', TurtleComponent);
}