use log::info;
use minecraft_assets::api::AssetPack;
use minecraft_assets::api::ResourceKind::BlockStates;
use minecraft_assets::schemas::BlockStates::Multipart;
use rocket::{Config, data, tokio};
use rocket::form::Form;
use rocket::fs::{FileServer, NamedFile};
use rocket::response::stream::EventStream;
use serde::{Deserialize, Serialize};
use std::thread::ThreadId;
use std::{collections::HashMap, path::PathBuf, path::Path, vec, fs};
use std::sync::{Arc, Mutex};
use uuid::{Uuid};
use rocket::{http::Status, serde::json, State};
use rocket::request::{FromRequest, Request, Outcome};
use rocket::tokio::sync::mpsc;
use rocket::tokio::time::{Duration, interval, timeout};
use rocket_ws as ws;
mod chunks;
mod turtle_data;
mod coordinates;
mod astar;
mod data_extractor;
use chunks::{Chunk, BlockData, WhitelistMap};
use coordinates::Coordinate;
use turtle_data::{Turtle, Slot, Inventory};
use sha_file_hashing::Hashable;
use astar::pathfind;

use crate::chunks::BlockStateData;
#[macro_use] extern crate rocket;


#[derive(Debug)]
enum ApiKeyError {
    Missing,
    Invalid,
}

struct ApiKey {
    uuid: uuid::Uuid
}

impl ApiKey {
    // Creates a new UUID for the API key and saves the file
    fn new() -> Self {
        let new_uuid = Uuid::new_v4();
        fs::write("api_key.txt", new_uuid.to_string()).expect("Should be able to write to `api_key.txt`");

        Self {
            uuid: new_uuid
        }
    }

    // Creates a new UUID object from the file
    fn load() -> Self {
        let  data = fs::read_to_string("api_key.txt").expect("Should be able to read `api_key.txt`");
        return ApiKey { uuid: Uuid::parse_str(&data).unwrap() };
    }

    // Either loads the file or creates a new UUID if there isnt one
    fn load_or_new() -> Self {
        if Path::new("api_key.txt").exists() {
            return Self::load();
        } else {
            Self::new()
        }
    }

    fn equal_to_string(&self, check_with: &str) -> bool {
        if Uuid::parse_str(check_with).is_err() {
            return false
        }

        self.uuid == Uuid::parse_str(check_with).unwrap()
    }
}

// Data structured so the turtle can read and parse it
// Also the data structure sent from the turtle
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
struct TurtleReadable {
    instruction: String,
    data: String
}

impl TurtleReadable {
    fn new(instruction: &str, data: &str) -> Self {
        TurtleReadable { instruction: instruction.to_string(), data: data.to_string() }
    }

    fn serialize(self) -> json::Json<TurtleReadable> {
        json::Json(self)
    }

    fn to_ws_message(self) -> ws::Message {
        ws::Message::Text(json::to_string(&self).unwrap())
    }
}

// NOTE: This function is partially created with AI :(
// Registry that maps a turtle's id to a sender half of an mpsc channel.
// Any route (e.g. web_command) can grab this shared, managed state and push
// a message onto a specific turtle's channel. The websocket task for that
// turtle is the one reading from the *receiver* half and forwarding
// the message out over the actual websocket.
struct TurtleConnections {
    senders: Mutex<HashMap<u16, mpsc::UnboundedSender<ws::Message>>>
}

impl TurtleConnections {
    fn new() -> Self {
        TurtleConnections { senders: Mutex::new(HashMap::new()) }
    }

    fn register(&self, id: u16, sender: mpsc::UnboundedSender<ws::Message>) {
        self.senders.lock().unwrap().insert(id, sender);
    }

    fn unregister(&self, id: u16) {
        self.senders.lock().unwrap().remove(&id);
    }

    // Returns true if the message was successfully queued for delivery
    fn send_to(&self, id: u16, message: ws::Message) -> bool {
        if let Some(sender) = self.senders.lock().unwrap().get(&id) {
            sender.send(message).is_ok()
        } else {
            false
        }
    }


    fn get_connected_ids(&self) -> Vec<u16> {
        let senders_vec = self.senders.lock().unwrap().keys().copied().collect();
        senders_vec
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `key` is a valid API key
        fn is_valid(key: &str) -> bool {
            ApiKey::load().equal_to_string(key)
        }

        match req.headers().get_one("api_key") {
            None => Outcome::Error((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(ApiKey::load()),
            Some(_) => Outcome::Error((Status::BadRequest, ApiKeyError::Invalid)),
        }
    }
}

// Turns a string like "dlllrrrruudr" into "dl3r4u2dr"
fn run_length_encode_string(input: &String) -> String {
    let mut final_string = "".to_string();

    let mut chars = input.chars();

    let mut last_char = chars.next().unwrap();

    let mut counter = 1;

    for (pos, char) in chars.enumerate() {
        if last_char == char {
            counter += 1
        } else {
            if counter == 1 {
                final_string.push_str(&last_char.to_string());
            } else {
                final_string.push_str(&format!("{}{}",last_char,counter));
            }
            counter = 1
        }

        if pos == input.len() - 2 {
            if counter == 1 {
                final_string.push_str(&char.to_string());
            } else {
                final_string.push_str(&format!("{}{}",last_char,counter));

            }
        }

        last_char = char;
    }

    final_string
}

// The string this outputs should be able to be sent to the turtle as movementPath command data
// If it cant find a path it returns none
fn get_path(whitelist: WhitelistMap, from: Coordinate, to: Coordinate, turtle_facing: &String) -> Option<String> {
    let path = pathfind(&from, &to, whitelist);
    let mut turtle_direction = turtle_facing.as_str();

    match path {
        // A path was found
        Some(path_cost) => {
            let mut instructions = "".to_string();

            // path_cost.0 is the path formatted as a bunch of (basically) coordinates as X, Y and Z, path_cost.1 is the calculated cost of the path, dependent on the astar algorithm cost function + path length
            let pos_list = &path_cost.0;
            for i in 1..pos_list.len() {
                // We start at 1 because the start of the path is where the turtle already is
                let last = &pos_list[i-1];
                let current = &pos_list[i];

                let direction_moved = (current.0 - last.0, current.1 - last.1, current.2 - last.2, turtle_direction);

                // This match statement essentially turns a list of coordinates into a list of moves from the POV of the turtle
                // Key
                // u = up, d = down, f = forward, r = turn right, l = turn left
                let direction_moved: (&str, &str) = match direction_moved {
                    (0, 1, 0, _) => ("u",turtle_direction),
                    (0, -1, 0, _) => ("d",turtle_direction),

                    (-1, 0, 0, "w") => ("f","w"),
                    (-1, 0, 0, "n") => ("lf","w"),
                    (-1, 0, 0, "s") => ("rf","w"),
                    (-1, 0, 0, "e") => ("llf","w"),

                    (1, 0, 0, "e") => ("f","e"),
                    (1, 0, 0, "s") => ("lf","e"),
                    (1, 0, 0, "n") => ("rf","e"),
                    (1, 0, 0, "w") => ("llf","e"),

                    (0, 0, -1, "n") => ("f","n"),
                    (0, 0, -1, "e") => ("lf","n"),
                    (0, 0, -1, "w") => ("rf","n"),
                    (0, 0, -1, "s") => ("llf","n"),

                    (0, 0, 1, "s") => ("f","s"),
                    (0, 0, 1, "w") => ("lf","s"),
                    (0, 0, 1, "e") => ("rf","s"),
                    (0, 0, 1, "n") => ("llf","s"),

                    _ => ("ERR", "ERR")
                };

                turtle_direction = direction_moved.1;

                instructions.push_str(direction_moved.0);
            }

            Some(run_length_encode_string(&instructions))
        }

        // No path could be found
        None    => {
            println!("No path found from {:?} to {:?}!", from, to);
            None
        }
    }
}

// Registers a turtle's data, used to update turtle data files currently
fn ws_register(reg_data: &String, connections: &Arc<TurtleConnections>, turtle_id: u16) {
    let reg_data: Turtle = json::from_str(&reg_data).unwrap();

    reg_data.save(TURTLES_FOLDER.into());
    let response = TurtleReadable::new("status", "successful").to_ws_message();
    connections.send_to(turtle_id, response);
}

// Recieves blocks from the turtles to be stored in chunk files
fn ws_receive_blocks(data: &String) {
    let blocks: Vec<(BlockData, Coordinate)> = json::from_str(&data).unwrap();

    for block in blocks.iter() {
        let world_coords = Coordinate::new(block.1.x, block.1.y, block.1.z);
        let chunk_coords = world_coords.world_to_chunk_coords();

        let mut chunk = Chunk::load_or_new(&WORLD_FOLDER, &chunk_coords);
        let local_coords = world_coords.world_to_local_coords();

        chunk.set_block(&local_coords, &BlockData { name: block.0.name.clone(), states: block.0.states.clone() });
        chunk.save(&WORLD_FOLDER);
    }
}

fn ws_send_lua_file(file_name: &String, connections: &Arc<TurtleConnections>, turtle_id: u16) -> bool {
    let file_data = fs::read_to_string(LUA_FOLDER.to_owned()+"/"+file_name);

    let Ok(file_data) = file_data else {
        println!("Couldn't read file path {:?}", file_data);
        return false;
    };

    let mut send_data = HashMap::new();

    send_data.insert("file_name", file_name);
    send_data.insert("content", &file_data);

    let send_data_serialized = json::to_string(&send_data).unwrap();

    connections.send_to(turtle_id, TurtleReadable::new("fileData", &send_data_serialized).to_ws_message());
    return true;
}

fn resolve_safe_path(file_name: &str) -> Option<PathBuf> {
    let base = Path::new(LUA_FOLDER).canonicalize().ok()?;
    let candidate = base.join(file_name);

    // canonicalize() resolves the path + requires it to exist
    // This also collapses any ".." components against the real filesystem
    let canonical = candidate.canonicalize().ok()?;

    if canonical.starts_with(&base) {
        Some(canonical)
    } else {
        None
    }
}

// This might be unsafe as people may be able to hash any file on the system based on the way im handing the path
fn ws_verify_file(data: &String, connections: &Arc<TurtleConnections>, turtle_id: u16) {
    let data: (String, String) = json::from_str(&data).unwrap();
    let file_name = data.0;
    let file_hash = data.1;

    let Some(server_file_path) = resolve_safe_path(&file_name) else {
        connections.send_to(turtle_id, TurtleReadable::new("fileNotFound", &file_name.to_string()).to_ws_message());
        return
    };

    let server_file_hash = server_file_path.hash();

    // Verify that the message is ok
    let Ok(server_file_hash) = server_file_hash else {
        connections.send_to(turtle_id, TurtleReadable::new("fileNotFound", &file_name.to_string()).to_ws_message());
        return
    };

    // Either tells the turtle the file is fine or tells it to re-download the file
    if server_file_hash == file_hash {
        connections.send_to(turtle_id, TurtleReadable::new("fileIdentical", &file_name.to_string()).to_ws_message());
    } else {
        ws_send_lua_file(&file_name.to_string(), connections, turtle_id);
    }
}

// NOTE: This function is partially created with AI :(
// Starts a websocket connection with a turtle.
// Turtles connect with their id in the query string, e.g. `/websocket?id=5`
// We register an mpsc sender for that id in the shared TurtleConnections state, then run two loops concurrently:
//   - outgoing: anything pushed onto the mpsc channel (e.g. from web_command)
//     gets forwarded out over the actual websocket to the turtle
//   - incoming: anything the turtle sends back gets read and can be handled
//     (logged, parsed, used to update turtle state, etc.)
#[get("/websocket?<id>")]
fn websocket(ws: ws::WebSocket, id: u16, connections: &State<Arc<TurtleConnections>>, _key: ApiKey) -> ws::Channel<'static> {
    use rocket::futures::{SinkExt, StreamExt};

    info!("Turtle id {} is connecting.",id);

    // Loads this turtle to edit its data
    let this_turtle = Turtle::load(TURTLES_FOLDER.into(), id);
    match this_turtle {
        Some(mut turtle) => {
            // Sets its component
            turtle.connected = true;
            // Saves it back to the file
            turtle.save(TURTLES_FOLDER.into());
        }
        _ => {
            warn!("Couldn't modify {}.json as connected! Did it register?",id);
        }
    }


    let connections = connections.inner().clone();
    let (tx, mut rx) = mpsc::unbounded_channel::<ws::Message>();
    connections.register(id, tx);

    ws.channel(move |stream| Box::pin(async move {
        let (mut sink, mut source) = stream.split();

        let outgoing = async {
            while let Some(msg) = rx.recv().await {
                if sink.send(msg).await.is_err() {
                    break;
                }
            }
        };

        let incoming = async {
            loop {
                let message = source.next().await;

                // Verify that the message is ok
                let Some(Ok(message)) = message else {
                    break
                };

                // Makes sure that it is a text input
                let ws::Message::Text(message) = message else {
                    // Unexpected result, we just ignore it
                    println!("Recieved unexpected websocket result. Ignoring.");
                    continue
                };

                // Deserializes the json into a TurtleReadable object
                // It is likely the case that message.data is another json string, which we can then decode in the respective function
                let message: Result<TurtleReadable, json::serde_json::Error> = json::from_str(&message);

                // We make sure that the json deserialized properly
                match message {
                    Ok(message) => {
                        let _ = match message.instruction.as_str()  {
                        "register" => ws_register(&message.data, &connections, id),
                        "sendBlocks" => ws_receive_blocks(&message.data),
                        "verifyFile" => ws_verify_file(&message.data, &connections, id),
                        "ping" => {},

                        // Unexpected result, we just ignore it
                        _ => {
                            println!("Recieved unknown websocket result. Ignoring.");
                            continue
                        }
                        };
                    }

                    Err(_) => println!("Error parsing json. Ignoring.")
                }
            }
        };


        rocket::tokio::select! {
            _ = outgoing => {},
            _ = incoming => {},
        }

        info!("Turtle id {} is disconnecting.",id);

        // Loads this turtle to edit its data
        let this_turtle = Turtle::load(TURTLES_FOLDER.into(), id);

        match this_turtle {
            Some(mut turtle) => {
                // Sets its component
                turtle.connected = false;
                // Saves it back to the file
                turtle.save(TURTLES_FOLDER.into());
            }
            _ => {
                warn!("Couldn't modify {}.json as disconnected! Did it EVER register?",id);
            }
        }

        connections.unregister(id);

        Ok(())
    }))
}

// Checks each turtle's file and sets it to disconnected
fn prune_turtles() {
    let registered_turtles = fs::read_dir(TURTLES_FOLDER).unwrap();

    let turtle_list = registered_turtles
        .filter_map(|f| f.ok())
        .filter_map(|f|Some(f.file_name()))
        .filter_map(|f|Path::new(&f)
        .file_stem()
        .map(|f|f.to_string_lossy().to_string().to_owned()));

    for turtle in turtle_list {
        let mut new_turtle = Turtle::load(TURTLES_FOLDER.into(), turtle.parse().unwrap()).unwrap();
        new_turtle.connected = false;
        new_turtle.save(TURTLES_FOLDER.into());
    }
}

// Handles the front page
#[get("/")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("frontend/front_page.html").await
}

// Handles serving the favicon
#[get("/favicon.ico")]
async fn serve_favicon() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("frontend/resources/images/favicon.ico").await
}

// Handles the control test page
#[get("/control")]
async fn control() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("frontend/control_test.html").await
}

// Handles the control test page
#[get("/comptest")]
async fn component_test() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("frontend/component_test.html").await
}

#[derive(FromForm, Debug)]
struct WebCommand<'r> {
    id: u16,
    kind: &'r str,
    data: &'r str,
}

// Forwards a form submission to the specific turtle's open websocket connection, if one exists.
#[post("/web_command", data = "<command>")]
fn web_command(command: Form<WebCommand<'_>>, connections: &State<Arc<TurtleConnections>>) -> Status {

    let message = TurtleReadable::new(command.kind, command.data).to_ws_message();

    if connections.send_to(command.id, message) {
        Status::Ok
    } else {
        // No open websocket for that turtle id
        Status::NotFound
    }
}

// We send back json containing data the user may need to manage turtles
// TODO: Make this live updating in the future
#[get("/connected_ids")]
fn connected_ids(connections: &State<Arc<TurtleConnections>>) -> json::Json<Vec<u16>> {
    let connections = connections.get_connected_ids();
    json::Json(connections)
}

// We send back json containing data the user may need to manage turtles
// Unlike connected ids, this function returns every id that has ever registered with the server
#[get("/all_ids")]
fn all_ids() -> json::Json<Vec<String>> {
    let registered_turtles = fs::read_dir(TURTLES_FOLDER).unwrap();

    let turtle_list = registered_turtles
        .filter_map(|f| f.ok())
        .filter_map(|f|Some(f.file_name()))
        .filter_map(|f|Path::new(&f)
        .file_stem()
        .map(|f|f.to_string_lossy().to_string().to_owned()))
        .collect();

    json::Json(turtle_list)
}

// We send back json containing chunk data
#[get("/get_chunk/<x>/<y>/<z>")]
fn get_chunk(x: i32, y: i32, z: i32) -> Option<json::Json<Chunk>> {
    let chunk = Chunk::load(&WORLD_FOLDER, &coordinates::Coordinate::new(x, y, z));

    chunk.map(|chunk|json::Json(chunk))
}

#[get("/get_available_chunks")]
fn get_available_chunks() -> json::Json<Vec<Coordinate>> {
    let available_chunks = fs::read_dir(&WORLD_FOLDER).unwrap();

    let chunk_list: Vec<String> = available_chunks
        .filter_map(|f| f.ok())
        .filter_map(|f|Some(f.file_name()))
        .filter_map(|f|Path::new(&f)
        .file_stem()
        .map(|f|f.to_string_lossy().to_string().to_owned()))
        .collect();

    let chunk_list: Vec<Coordinate> = chunk_list.iter().filter_map(|f| {
        match f.as_str() {
            "whitelist" => None,
            _ => {
                let coords: Vec<i32> = f.split("_").map(|f|f.parse().unwrap()).collect();
                Some(Coordinate::new(coords[0], coords[1], coords[2]))
            }
        }
    }).collect();

    json::Json(chunk_list)
}

const LUA_FOLDER: &str = "lua";
const WORLD_FOLDER: &str = "world_data";
const TURTLES_FOLDER: &str = "turtles";
const FRONTEND_FOLDER: &str = "frontend";
const TEMP_THREEJS_FOLDER: &str = "threejs";
const EXTRACTED_DATA_FOLDER: &str = "extracted_minecraft_data";
const MINECRAFT_DATA_FOLDER: &str = "minecraft_data";

#[launch]
fn rocket() -> _ {
    let data_extractor = data_extractor::MCDataCrawler::new(MINECRAFT_DATA_FOLDER.into(), EXTRACTED_DATA_FOLDER.into());
    data_extractor.extract_data();
    let model_loader = data_extractor::ModelLoader::new((EXTRACTED_DATA_FOLDER.to_owned()+"/").into());
    let tst_data = Chunk::load(&WORLD_FOLDER, &Coordinate { x: -8, y: 4, z: 7}).unwrap();
    let tst_block_data = &tst_data.block_data[4][5][9];
    let model_properties = model_loader.get_model_props(tst_block_data);
    dbg!(&model_properties);
    let test_model = model_loader.get_model_render(model_properties.unwrap());
    dbg!(test_model);

    // Creates the world data folder if it doesnt exist
    let path = PathBuf::from(WORLD_FOLDER);
    let _ = fs::create_dir(&path);

    let whitelist = WhitelistMap::load_or_new(&path.join("whitelist"));
    whitelist.save().unwrap();

    // Sets all registered turtles to be marked as disconnected
    prune_turtles();

    // Creates a new API key if there isn't one
    ApiKey::load_or_new();

    rocket::build()
    // Initializes the turtle connection manager
    .manage(Arc::new(TurtleConnections::new()))
    // This hosts all the files in the lua folder, so if we recieve a get request that has /lua/filepath it will go to that filepath
    .mount("/".to_owned()+LUA_FOLDER, FileServer::from(LUA_FOLDER.to_owned()+"/"))
    // This hosts all the turtle data for easy frontend access
    .mount("/".to_owned()+TURTLES_FOLDER, FileServer::from(TURTLES_FOLDER.to_owned()+"/"))
    // This hosts all the files in the frontend folder
    .mount("/".to_owned()+FRONTEND_FOLDER, FileServer::from(FRONTEND_FOLDER.to_owned()+"/"))
    // This hosts all the files in the threejs folder
    .mount("/".to_owned()+TEMP_THREEJS_FOLDER, FileServer::from(TEMP_THREEJS_FOLDER.to_owned()+"/"))
    // This hosts all the texture data for easy frontend access
    .mount("/".to_owned()+EXTRACTED_DATA_FOLDER, FileServer::from(EXTRACTED_DATA_FOLDER.to_owned()+"/"))

    .mount("/", routes![
        websocket,
        index,
        control,
        web_command,
        connected_ids,
        serve_favicon,
        component_test,
        all_ids,
        get_chunk,
        get_available_chunks
        ])
}

// TODO:
// Implement pings on the rust side to make sure the connection is active
// See if you can move over the pathfinding and world exploration code from the turtleswarm project
// Add a login system so only people who are authorized can send commands to turtles (maybe)
// Add world file importing
// Add a system so turtles can hash their files to determine if they are out of date