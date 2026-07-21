use file_crawler::prelude::rayon::vec;
use lz4_flex::block;
use serde::{Deserialize, Serialize};
use std::{collections::{HashMap, HashSet}, fs, path::{Path, PathBuf}};
use rocket::{form::name, serde::json::{self, Json, Value}};
use rocket::serde::json::serde_json::json;
use crate::coordinates::Coordinate;
use schematic_mesher::{BlockSource, BlockPosition, BoundingBox, InputBlock};

#[derive(Debug, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Chunk {
    pub coordinates: Coordinate,
    pub block_data: Vec<Vec<Vec<BlockData>>>
}

impl Chunk {
    /// Creates a 16x16x16 vector filled with air
    pub fn new(coordinates: &Coordinate) -> Self {
        Self {
            coordinates: *coordinates,
            block_data: vec![vec![vec![BlockData {name: "minecraft:air".to_string(), states: HashMap::new() }; 16]; 16]; 16]
        }
    }

    /// Sets a block in the chunk to the input
    pub fn set_block(&mut self, coordinate: &Coordinate, block: &BlockData) {
        self.block_data[coordinate.x as usize][coordinate.y as usize][coordinate.z as usize] = block.clone();
    }

    pub fn get_name(coords: &Coordinate) -> String {
       coords.x.to_string() + "_" + &coords.y.to_string() + "_" + &coords.z.to_string()
    }

    /// Saves this chunks data to the given path with the correct name
    pub fn save<P: AsRef<Path>>(&self, path: &P) {
        let path = path.as_ref();
        let file_name = Self::get_name(&self.coordinates);
        let chunk_file = std::fs::File::create(path.join(file_name)).unwrap();

        let chunk_file = lz4_flex::frame::FrameEncoder::new(chunk_file).auto_finish();

        ciborium::into_writer(self, chunk_file).unwrap();
    }

    /// Creates a new chunk object from a path that is given
    pub fn load<P: AsRef<Path>>(path: &P, coordinates: &Coordinate) -> Option<Self> {
        let path = path.as_ref();
        let file_name = Self::get_name(coordinates);
        let reader = std::fs::File::open(path.join(file_name)).ok()?;

        let reader = lz4_flex::frame::FrameDecoder::new(reader);

        ciborium::from_reader(reader).unwrap()
    }

    pub fn load_or_new<P: AsRef<Path>>(path: &P, coordinates: &Coordinate) -> Self {
        let path = path.as_ref();
        if let Some(chunk) = Self::load(&path, coordinates){
            chunk
        } else {
            Self::new(coordinates)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(untagged)]
pub enum BlockStateData {
    Bool(bool),
    Integer(i32),
    String(String),
}

impl ToString for BlockStateData {
    fn to_string(&self) -> String {
        match self {
            BlockStateData::Bool(bool) => {
                bool.to_string()
            }
            BlockStateData::Integer(integer) => {
                integer.to_string()
            }
            BlockStateData::String(string) => {
                string.clone()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct BlockData {
    pub name: String,
    pub states: HashMap<String, BlockStateData>
}

// The whitelist map is used to tell the pathfinder functions what blocks turtles are allowed to move through
#[derive(Debug, Clone)]
#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct WhitelistMap {
    pub map: HashSet<String>,

    #[serde(skip)]
    pub path: PathBuf
}

impl WhitelistMap {
        /// Makes a new WhitelistMap
        pub fn new<P: AsRef<Path>>(path: &P) -> Self {
            let path = path.as_ref().to_path_buf();
            let mut map = HashSet::new();
            map.insert("minecraft:air".to_string());
            Self {path, map}
        }

        pub fn load<P: AsRef<Path>>(path: &P) -> Self {
            let path = path.as_ref();

            let mut temp_self: Self = json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
            temp_self.path = path.to_path_buf();
            temp_self
        }

        pub fn load_or_new<P: AsRef<Path>>(path: &P) -> Self {
            let path = path.as_ref();
            if path.exists() {
                Self::load(&path)
            } else {
                Self::new(&path)
            }
        }

        pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
            _ = fs::write(&self.path, json::to_pretty_string(self).unwrap());
            Ok(())
        }
}