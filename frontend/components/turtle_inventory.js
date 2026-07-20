import { Slot } from "/frontend/components/inventory_slot.js";

export class Inventory {
    constructor(size, slot_list) {
        this.size = size;
        this.slots = [];

        for (let i = 0; i < size; i ++) {
            if (slot_list[i] == null) {
                this.slots.push(new Slot("Empty", 0))
            } else {
                this.slots.push(new Slot(slot_list[i].name, slot_list[i].count))
            }
        }
    }
}

class TurtleInventoryComponent extends HTMLElement {
    connectedCallback() {
        for (let i = 0; i < 16; i ++) {
            let new_slot = document.createElement("x-inventory-slot")
            // We do i+1 here as computercraft turtle inventories start at index 1
            new_slot.id = i+1
            this.appendChild(new_slot);
        }

        this.update();
    }

    #inv = Inventory
    set contents(newContents) {
        this.#inv = newContents;
        this.update();
    }

    // TODO: Implement inventory
    update() {
        // for (const [i, v] of this.childNodes.entries()) {
        //     if (this.#inv[i]) {
        //         v.name = this.#inv[i].name
        //         v.count = this.#inv[i].count

        //         if (this.#inv[i].name !="Empty") {
        //             let name_and_space = this.#inv[i].name.split(":")
        //             let space = name_and_space[0]
        //             let name = name_and_space[1]
        //             v.image = "/extracted_minecraft_data/"+space+"/"+name+".png"
        //         }
        //     }
        // }
    }
}

export const registerTurtleInventoryComponent = () => {
    customElements.define('x-turtle-inventory', TurtleInventoryComponent);
}