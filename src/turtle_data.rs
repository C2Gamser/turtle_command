use std::{ffi::OsString, fs, convert::From};

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
                let mut new_inventory = Inventory {size: size, slots: (contents) };
                let deficit = new_inventory.size - (new_inventory.slots.len() as u32);

                if deficit > 0 {
                    // Fills the rest of the array with None if it isnt full

                    let mut slice: Vec<Option<Slot>> = vec![None;deficit as usize];
                    new_inventory.slots.append(&mut slice);
                }
                new_inventory
            }

            None => Self::new_empty(size)
        }
    }

    pub fn new_empty(size: u32) -> Self {
        Inventory { size: size, slots: vec![None; size.try_into().unwrap()] }
    }
}

impl From<(u32, Option<Vec<Option<Slot>>>)> for Inventory {
    fn from(inventory_precursor: (u32, Option<Vec<Option<Slot>>>)) -> Self {
        match inventory_precursor.1 {
            Some(contents) => Inventory { size: inventory_precursor.0, slots: contents},
            None => Self::new_empty(inventory_precursor.0)
        }

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
    // Saves itself to a file in turtles/ with the name being its id
    pub fn save(&self) {
        let string_self = json::to_pretty_string(&self).unwrap();
        if !fs::exists("turtles/").unwrap() {
            fs::create_dir("turtles/").unwrap();
        }
        fs::write(format!("turtles/{}.json",self.id), string_self).expect(&format!("Should be able to write to `turtles/{}.json`",self.id));
    }

    pub fn load(filepath: OsString) -> Self {
        let  data = fs::read_to_string(&filepath).expect(&format!("Should be able to read `{}`",filepath.display()));
        let new_self: Turtle = json::from_str(&data).unwrap();

        new_self
    }
}