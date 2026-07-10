use serde::Serialize;
use std::{mem::transmute, path::Path};
use uuid::{Uuid, uuid};
use std::fs;

use rocket::{http::Status, serde::json::Json};
use rocket::request::{FromRequest, Request, Outcome};
#[macro_use] extern crate rocket;

#[derive(Serialize)]
struct LuaReadableResponse {
    kind: String,
    payload: String
}

impl LuaReadableResponse {
    fn to_string(&self) -> String {
        format!("{} {}",self.kind,self.payload)
    }
}

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
        self.uuid == Uuid::parse_str(check_with).unwrap()
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

// Registers a turtle in the network
#[post("/register", data = "<registration_data>")]
fn register(registration_data: Json<String>, key: ApiKey) -> String {
    LuaReadableResponse { kind: "Response".to_string() }.to_string()
}

// Starts a connection with a turtle
#[get("/connect")]
fn connect(key: ApiKey) -> &'static str {
    "Hello"
}

#[launch]
fn rocket() -> _ {
    // Creates a new API key if there isn't one
    ApiKey::load_or_new();
    rocket::build().mount("/", routes![connect,register])
}