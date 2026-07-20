import { registerTurtleComponent } from '/frontend/components/turtle.js';
import { registerTurtleLoaderComponent } from '/frontend/components/turtle_loader.js';
import { registerTurtleInventoryComponent } from '/frontend/components/turtle_inventory.js';
import { registerInventorySlotComponent } from '/frontend/components/inventory_slot.js';

const app = () => {
    registerTurtleComponent();
    registerTurtleLoaderComponent();
    registerTurtleInventoryComponent();
    registerInventorySlotComponent();
}

document.addEventListener('DOMContentLoaded', app);