use std::fs;

use rocket::serde::json;

use crate::coordinates::Coordinate;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot {
    name: String,
    count: i8
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Inventory {
    pub size: u32,
    pub slots: Vec<Option<Slot>>
}

impl Inventory {
    pub fn new(size: u32, contents: Option<Vec<Option<Slot>>>) -> Self {
        match contents {
            Some(contents) => {
                Inventory {size: size, slots: (contents)}
            }

            None => Self::new_empty(size)
        }
    }

    pub fn new_empty(size: u32) -> Self {
        Inventory { size: size, slots: vec![None; size.try_into().unwrap()] }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Turtle {
    pub id: u16,
    pub connected: bool,
    pub inventory: Inventory,
    pub equipped_left: Option<Slot>,
    pub equipped_right: Option<Slot>,
    pub coordinates: Coordinate,
    pub fuel: i16
}

impl Turtle {
    // Saves itself to a file in path with the name being its id
    pub fn save(&self, path: String) {
        let string_self = json::to_pretty_string(&self).unwrap();
        let my_path = self.id.to_string()+".json";

        fs::write(path+"/"+&my_path, string_self).expect(&format!("Should be able to write to `turtles/{}.json`",self.id));
    }

    // Loads a turtle from the given turtle folder using the id
    pub fn load(path: String, id: u16) -> Option<Self> {
        let turtle_path = path+"/"+&id.to_string()+".json";

        let data = fs::read_to_string(&turtle_path);

        match data {
            Ok(string) => {
                let new_self: Turtle = json::from_str(&string).unwrap();
                Some(new_self)
            }
            Err(_) => {
                None
            }
        }
    }
}