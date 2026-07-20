export class Slot {
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

/**
 * Usage:
 * <x-inventory-slot id=number></x-inventory-slot>
 * where number is slot's id, aka location in the inventory (for turtles that would be 1-16)
 */
class InventorySlotComponent extends HTMLElement {
    connectedCallback() {
        // Add an image in here eventually
    }

    #name = ""
    #count = 0
    #src = ""
    set name(newName) {
        this.#name = newName;
        this.update();
    }

    set count(newCount) {
        this.#count = newCount;
        this.update();
    }

    set image(newImageSource) {
        this.#src = newImageSource;
        console.log(this.#src)
        this.update();
    }

    update() {
        if (this.#count > 0) {
            this.title = this.#name+" x"+this.#count
        } else {
            this.title = this.#name
        }
    }
}

export const registerInventorySlotComponent = () => {
    customElements.define('x-inventory-slot', InventorySlotComponent);
}