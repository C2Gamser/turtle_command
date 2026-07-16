use rocket::{Config, tokio};
use rocket::form::Form;
use rocket::fs::{FileServer, NamedFile};
use rocket::response::stream::EventStream;
use serde::{Deserialize, Serialize};
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
use chunks::{Chunk, BlockData, WhitelistMap};
use coordinates::Coordinate;
use turtle_data::{Turtle, Slot, Inventory};
use astar::pathfind;
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
fn ws_register(reg_data: &String, connections: &Arc<TurtleConnections>) {
    let reg_data: Turtle = json::from_str(&reg_data).unwrap();

    reg_data.save(TURTLES_FOLDER.into());
    let response = TurtleReadable::new("status", "successful").to_ws_message();
    connections.send_to(reg_data.id, response);
}

// Recieves blocks from the turtles to be stored in chunk files
fn ws_receive_blocks(reg_data: &String) {
    let blocks: Vec<(BlockData, Coordinate)> = json::from_str(&reg_data).unwrap();

    for block in blocks.iter() {
        let world_coords = Coordinate::new(block.1.x, block.1.y, block.1.z);
        let chunk_coords = world_coords.world_to_chunk_coords();

        let mut chunk = Chunk::load_or_new(&WORLD_FOLDER, &chunk_coords);
        let local_coords = world_coords.world_to_local_coords();

        chunk.set_block(&local_coords, &BlockData { name: block.0.name.clone(), states: block.0.states.clone() });
        chunk.save(&WORLD_FOLDER);
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

    let connections = connections.inner().clone();
    let (tx, mut rx) = mpsc::unbounded_channel::<ws::Message>();
    connections.register(id, tx);

    connections.send_to(id, TurtleReadable::new("keepAliveTime", "4").to_ws_message());

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
                let message = timeout(Duration::from_secs(8), source.next()).await;

                // If we timed out
                let Ok(message) = message else {
                    println!("Connection timed out with turtle ID {}, closing.",id);
                    connections.send_to(id, TurtleReadable::new("closingConnection", "closing").to_ws_message());
                    let mut close_timer = interval(Duration::from_secs(1));
                    close_timer.tick().await;
                    break
                };

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
                        "register" => ws_register(&message.data, &connections),
                        "sendBlocks" => ws_receive_blocks(&message.data),
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

        connections.unregister(id);

        Ok(())
    }))
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

const LUA_FOLDER: &str = "lua";
const WORLD_FOLDER: &str = "world_data";
const TURTLES_FOLDER: &str = "turtles";
const SCRIPTS_FOLDER: &str = "frontend/scripts";
const RESOURCE_FOLDER: &str = "frontend/resources/";

#[launch]
fn rocket() -> _ {
    // Creates the world data folder if it doesnt exist
    let path = PathBuf::from(WORLD_FOLDER);
    let _ = fs::create_dir(&path);

    let whitelist = WhitelistMap::load_or_new(&path.join("whitelist"));
    whitelist.save().unwrap();

    // Creates a new API key if there isn't one
    ApiKey::load_or_new();

    rocket::build()
    // Initializes the turtle connection manager
    .manage(Arc::new(TurtleConnections::new()))
    // This hosts all the files in the lua folder, so if we recieve a get request that has /lua/filepath it will go to that filepath
    .mount("/".to_owned()+LUA_FOLDER, FileServer::from(LUA_FOLDER.to_owned()+"/"))
    // This hosts the files in the scripts folder for easy script frontend access
    .mount("/".to_owned()+SCRIPTS_FOLDER, FileServer::from(SCRIPTS_FOLDER.to_owned()+"/"))
    // This hosts all the turtle data for easy frontend access
    .mount("/".to_owned()+TURTLES_FOLDER, FileServer::from(TURTLES_FOLDER.to_owned()+"/"))
    // This hosts all the files in the resources folder for easy frontend access
    .mount("/".to_owned()+RESOURCE_FOLDER, FileServer::from(RESOURCE_FOLDER.to_owned()+"/"))

    .mount("/", routes![websocket, index, control, web_command, connected_ids, serve_favicon])
}

// TODO:
// Implement pings on the rust side to make sure the connection is active
// See if you can move over the pathfinding and world exploration code from the turtleswarm project
// Add a login system so only people who are authorized can send commands to turtles (maybe)
// Add world file importing