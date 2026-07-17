class TurtleLoaderComponent extends HTMLElement {
    connectedCallback() {
        this.update();
    }

    update() {
        // Fetches the connected turtles
        // Gets a list of connected turtle ids
        fetch("/connected_ids")
        .then((response) => response.json())
        .then((data) => {

            this.innerHTML = " "

            let timeout_tracker = 100
            for (const turtle_id of data) {
                let new_turt = document.createElement("x-turtle");
                new_turt.setAttribute("turtle_id", turtle_id);

                setTimeout(() => {
                    this.appendChild(new_turt);
                }, timeout_tracker);

                timeout_tracker += 100;
            };
        });
    }
}

export const registerTurtleLoaderComponent = () => {
    customElements.define('x-turtle-loader', TurtleLoaderComponent);
}