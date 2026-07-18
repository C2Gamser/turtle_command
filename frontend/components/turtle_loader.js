import { MultiAudio } from "/frontend/scripts/click_audio.js";

class TurtleLoaderComponent extends HTMLElement {
    connectedCallback() {
        this.update();
    }

    update() {
        // Gets a list of all turtle ids that have ever registered
        fetch("/all_ids")
        .then((response) => response.json())
        .then((data) => {
            this.innerHTML = " "
            // Manages click audio for all loaded turtles
            let click_audio = new MultiAudio("/frontend/resources/audio/Click_stereo.ogg", 8)

            // We use a mutation observer to make sure the content is loaded before applying event listeners for clicks
            // Sets up an observer config
            const observer_config = {childList:true}
            // Callback function to execute when mutations are observed
            const mutation_observer_callback = (mutationList, observer) => {
                // For each mutation (should only be in childlist due to config)
                for (const mutation of mutationList) {
                    // Look over each node added
                    for (const addedNode of mutation.addedNodes) {
                        if (addedNode.nodeName == "BUTTON") {
                            addedNode.addEventListener("click", function () {
                                click_audio.play()
                            });
                        }
                    }
                }
                // Disconnect so we don't bog down performance
                observer.disconnect();
            };

            // Tracks ms for spawning each turtle for cool animations
            let animation_tracker = 100
            for (const turtle_id of data) {
                let new_turt = document.createElement("x-turtle");
                new_turt.setAttribute("turtle_id", turtle_id);
                new_turt.setAttribute("live_update", true);

                // This is what tracks the turtle to add event listeners once the turtle's js is run
                const mutation_observer = new MutationObserver(mutation_observer_callback);
                mutation_observer.observe(new_turt, observer_config)

                setTimeout(() => {
                    this.appendChild(new_turt);
                }, animation_tracker);

                animation_tracker += 100;
            };
        });
    }
}

export const registerTurtleLoaderComponent = () => {
    customElements.define('x-turtle-loader', TurtleLoaderComponent);
}