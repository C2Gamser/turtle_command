import { Inventory } from "/frontend/components/turtle_inventory.js";
import { Slot } from "/frontend/components/inventory_slot.js";

class Coordinates {
    constructor(x, y, z) {
        this.x = x;
        this.y = y;
        this.z = z;
    }

    toString() {
        return `X:${this.x} Y:${this.y} Z:${this.z}`
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
            <img class="greenLight" src=/frontend/resources/images/turtle_green_light_off.png>
            <div class="turtleID"></div>
            <div class="coordinates hideOnExpand"></div>
            <div class="fuel hideOnExpand"></div>


            <form id="webCommandForm" class="showOnExpand" action="/web_command" method="post">
                <input type="hidden" name="id" value="${this.getAttribute("turtle_id")}" />

                <label for="kind">Command Kind:</label>
                <div class="terminalInput showOnExpand" class="showOnExpand">
                    >
                    <input autocomplete="off" value="" type="text" name="kind" required/>
                </div>
                <label for="data">Command Contents:</label>
                <div class="terminalInput showOnExpand" class="showOnExpand">
                    >
                    <input autocomplete="off" value="" type="text" name="data"/>
                </div>

                <input type="submit" hidden />
            </form>

            <div id="webCommandStatus" class="showOnExpand" status="none">test</div>

            <x-turtle-inventory></x-turtle-inventory>
        `;

        // Handle the form submission element to make it not go to a new page when submitting
        const turtle_form = this.querySelector("#webCommandForm");
        turtle_form.addEventListener("submit", handleFormSubmission);

        function handleFormSubmission(event) {
            console.info(`Form submitted to ${window.location.origin}/web_command`)

            fetch(`${window.location.origin}/web_command`, {
                method: "POST",
                body: JSON.stringify({
                    id: Number(this.elements["id"].value),
                    kind: this.elements["kind"].value,
                    data: this.elements["data"].value,
                })
            })
            // Sets the status message depending on if the request failed or worked
            .then((value) => {
                if (value.ok) {
                    let status_indicator = this.parentElement.querySelector("#webCommandStatus")
                    status_indicator.innerHTML = "Successfully sent."
                    status_indicator.style.opacity = "1";
                    status_indicator.style.color = "#2fd85c";
                    status_indicator.setAttribute("status", "ok");
                }

                if (!value.ok) {
                    let status_indicator = this.parentElement.querySelector("#webCommandStatus")
                    status_indicator.innerHTML = value.statusText;
                    status_indicator.style.opacity = "1";
                    status_indicator.style.color = "#d8402f";
                    status_indicator.setAttribute("status", "fail");
                }
            })

            // Turns the status message off after 1 second
            setTimeout(() => {
                let status_indicator = this.parentElement.querySelector("#webCommandStatus")
                status_indicator.setAttribute("status", "none")
            }, 1000);


            this.reset()

            event.preventDefault();
        }

        // Handles collapsing and showing the contents of the turtle
        this.getElementsByClassName("powerButton")[0].addEventListener("click", function (e) {
            if (this.parentElement.getAttribute("expanded") == "true") {
                this.parentElement.setAttribute("expanded", "false")
            } else {
                this.parentElement.setAttribute("expanded", "true")
            }
        });

        this.update();
        this.loop();
    }

    // Sets it up so the turtle auto fetches data every 2 seconds from the server
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
            let turtle_id_div = this.querySelector("div.turtleID")
            turtle_id_div.innerText = `Turtle ID: ${new_turt.id}`

            let coordinates_div = this.querySelector("div.coordinates")
            coordinates_div.innerText = `${new_turt.coordinates.toString()}`

            let fuel_div = this.querySelector("div.fuel")
            fuel_div.innerText = `Fuel: ${new_turt.fuel.toString()}`

            let turtle_inventory = this.querySelector("x-turtle-inventory")
            turtle_inventory.contents = new_turt.inventory.slots

            this.setAttribute("connected", new_turt.connected)
        });
    }
}

export const registerTurtleComponent = () => {
    customElements.define('x-turtle', TurtleComponent);
}