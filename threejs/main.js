import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera( 75, window.innerWidth / window.innerHeight, 0.1, 1000 );

scene.background = new THREE.Color().setHex( 0x67b7e6 )

let width = window.innerWidth
let height = window.innerHeight

const renderer = new THREE.WebGLRenderer();
const controls = new OrbitControls( camera, renderer.domElement );
controls.enableDamping = true
const mousePos = new THREE.Vector2()
renderer.setSize( width, height );
renderer.setAnimationLoop( animate );
document.body.appendChild( renderer.domElement );

const light = new THREE.AmbientLight( 0xffffff, 15 ); // white light
scene.add( light );

let rayCaster = new THREE.Raycaster();
let sceneMeshes = []

function getClicked3DPoint(evt) {
	evt.preventDefault();

	mousePos.set((evt.clientX / width) * 2 - 1, -(evt.clientY / height) * 2 + 1)

	rayCaster.setFromCamera(mousePos, camera);
	var intersects = rayCaster.intersectObjects(sceneMeshes, true);

	if (intersects.length > 0) {
		return intersects[0].point;
	}
};

function getTurtlePos(id) {
	fetch("/turtles/"+id+".json")
	.then((response) => response.json())
	.then((data) => {
		if (data.coordinates) {
			controls.target = new THREE.Vector3(data.coordinates.x, data.coordinates.y, data.coordinates.z);
			camera.position.set(data.coordinates.x+5, data.coordinates.y+10, data.coordinates.z+10);
			controls.update();
		}
      }
    );
}

const loader = new GLTFLoader();
function update_chunks(loader, get_all) {
	let selector = document.getElementById("turtleSelector")
	let cur_time = Date.now();
	// This will filter for all chunks as we set the "current_time" to 0
	if (get_all) {
		cur_time = 0
	}

	// Recieves a list of chunks (vec of coordinates) for all chunks updated past the given time
    fetch("/get_updated_chunks/"+cur_time)
	.then((response) => response.json())
	.then((data) => {
        for (const c of data) {
			loader.load(`/chunk_meshes/${c["x"]}_${c["y"]}_${c["z"]}.glb`, function ( gltf ) {
				let mesh = gltf.scene;
				mesh.position.set(c["x"]*16,c["y"]*16,c["z"]*16);
				mesh.name = `${c["x"]}_${c["y"]}_${c["z"]}`;
				scene.add(mesh);

				// Add to a list of total meshes so we can raycast on them
				sceneMeshes.push(mesh);
			}, undefined, function ( error ) {
				console.error( error );
			} );
		}
      }
    );
}

window.addEventListener('auxclick', (e) => {
	let clicked_point = getClicked3DPoint(e);
	if (clicked_point) {
		controls.target = clicked_point;
		controls.update();
	}
})

// responsive
function resize() {
  width = window.innerWidth
  height = window.innerHeight
  camera.aspect = width / height
  const target = new THREE.Vector3(0, 0, 0)
  const distance = camera.position.distanceTo(target)
  const fov = (camera.fov * Math.PI) / 180
  const viewportHeight = 2 * Math.tan(fov / 2) * distance
  const viewportWidth = viewportHeight * (width / height)
  camera.updateProjectionMatrix()
  renderer.setSize(width, height)
  scene.traverse((obj) => {
    if (obj.onResize) obj.onResize(viewportWidth, viewportHeight, camera.aspect)
  })
}

window.addEventListener('resize', resize)
resize()

// Get all chunk meshes stored on the server
update_chunks(loader, true)

// Every 10 seconds, only get new chunks
var chunk_update_interval = setInterval(function() {
	update_chunks(loader, false)
}, 10000);

getTurtlePos(0);

function animate() {
	// required if controls.enableDamping or controls.autoRotate are set to true
	controls.update();
	renderer.render( scene, camera );
}

