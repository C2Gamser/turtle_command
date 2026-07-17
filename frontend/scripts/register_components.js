import { registerTurtleComponent } from '/frontend/components/turtle.js';
import { registerTurtleLoaderComponent } from '/frontend/components/turtle_loader.js';

const app = () => {
    registerTurtleComponent();
    registerTurtleLoaderComponent();
}

document.addEventListener('DOMContentLoaded', app);