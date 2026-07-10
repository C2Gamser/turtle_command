#[macro_use] extern crate rocket;

// Registers a turtle in the network
#[get("/register")]
fn register() -> &'static str {
    "Regsiter!"
}

// Starts a connection with a turtle
#[get("/connect")]
fn connect() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![connect,register])
}

