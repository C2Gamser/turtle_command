use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Clone, Copy)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32
}

impl Coordinate {
    /// Creates a coordinate using the x, y, and z inputs
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {x, y, z}
    }

    pub fn world_to_local_coords(&self) -> Self {
        Coordinate::new(self.x & 0xF, self.y & 0xF, self.z & 0xF)
    }

    pub fn world_to_chunk_coords(&self) -> Self {
        Coordinate::new(self.x >> 4, self.y >> 4, self.z >> 4)
    }
}
