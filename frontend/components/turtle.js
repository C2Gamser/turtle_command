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
 * <x-turtle turtle_id=id></x-turtle>
 * where id is the turtle's id, e.g. 0
 */
class TurtleComponent extends HTMLElement {
    connectedCallback() {
        this.update();
    }

    update() {
        // Fetches the turtle's data
        fetch("/turtles/"+this.getAttribute("turtle_id")+".json")
        .then((response) => response.json())
        .then((data) => {
            let r = data;
            console.info(r)

            let new_turt = new Turtle(r)

            this.innerHTML = `
                <button class="powerButton"></button>
                <div id="turtle_id">Turtle ID: ${new_turt.id}</div>
                <div id="coordinates">${new_turt.coordinates.toString()}</div>
                <div id="fuel">Fuel: ${new_turt.fuel.toString()}</div>
            `;

            this.setAttribute("data-connected", new_turt.connected)

        });


    }
}

export const registerTurtleComponent = () => {
    customElements.define('x-turtle', TurtleComponent);
}