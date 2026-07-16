use std::{collections::BTreeMap, time::Instant};
use pathfinding::prelude::{astar};
use crate::chunks::{WhitelistMap, Chunk};
use crate::coordinates::Coordinate;

fn range_calculator(start: i32, radius: i32) -> std::ops::Range<i32> {
    (start-radius)..(start+radius)
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pos(pub i32, pub i32, pub i32);

impl Pos {
  fn distance(&self, other: &Pos) -> i16 {
    let dist = (self.0.abs_diff(other.0).pow(2) as f32 + self.1.abs_diff(other.1).pow(2) as f32 + self.2.abs_diff(other.2).pow(2) as f32).sqrt() as i16;
    //(self.0.abs_diff(other.0) + self.1.abs_diff(other.1) + self.2.abs_diff(other.2)) as i16
    dist
  }

  fn successors(&self, map: &BTreeMap<Pos, bool>) -> Vec<(Pos, i16)> {
    let mut successors:Vec<(Pos, i16)> = Vec::new();
    for offset_x in [-1i32, 0, 1] {
        for offset_y in [-1i32, 0, 1] {
            for offset_z in [-1i32, 0, 1] {
                if offset_x.abs()+offset_y.abs()+offset_z.abs() == 1 {
                    if ! map.contains_key(&Pos(self.0+offset_x, self.1+offset_y, self.2+offset_z)) {
                        successors.push((Pos(self.0+offset_x, self.1+offset_y, self.2+offset_z), 1));
                    }
                }
            }
        }
    }
    successors
    }
}

/// adds barriers to the chunk
fn add_chunk_to_map(chunk: &Chunk, map: &mut BTreeMap<Pos, bool>, whitelist: &WhitelistMap) {
    for row in chunk.block_data.iter().enumerate() {
        for column in row.1.iter().enumerate() {
            for block in column.1.iter().enumerate() {
                // if the whitelist has the current block id then dont add it to the barriers
                if whitelist.map.contains(&block.1.name) {
                    continue;
                }

                map.insert(Pos(
                    // Note: multiplying chunk coordinates by 32 turns it into global coordinates, then we add the coordinate in the chunk
                    (row.0 as i32 + chunk.coordinates.x*16) as i32,
                    (column.0 as i32 + chunk.coordinates.y*16) as i32,
                    (block.0 as i32 + chunk.coordinates.z*16) as i32),
                     false
                );
            }
        }
    }
}

// Note: Coordinates are global
pub fn pathfind(start: &Coordinate, goal: &Coordinate, whitelist: WhitelistMap) -> Option<(Vec<Pos>, i16)> {
    let start_chunk = start.world_to_chunk_coords();

    let mut map: BTreeMap<Pos, bool> = BTreeMap::new();
    const RADIUS: i32 = 5;

    // Note: this may take up a lot of time. Check this.
    for x in range_calculator(start_chunk.x, RADIUS) {
        for y in range_calculator(start_chunk.y, RADIUS) {
            for z in range_calculator(start_chunk.z, RADIUS) {
                let chunk = &Chunk::load_or_new(&crate::WORLD_FOLDER, &Coordinate::new(x, y, z));
                add_chunk_to_map(chunk, &mut map, &whitelist);
            }
        }
    }

    let start = Pos(start.x, start.y, start.z);
    let goal = Pos(goal.x, goal.y, goal.z);

    fn smart_goal(goal: &Pos, check_pos: &Pos, start_pos: &Pos) -> bool {
        if goal == check_pos {
            true
        } else {
            // Limits the max distance per astar path to 200 blocks travelled as otherwise it may take a long time to calculate
            if start_pos.0.abs_diff(check_pos.0) + start_pos.1.abs_diff(check_pos.1) + start_pos.2.abs_diff(check_pos.2) >= 200 {
                true
            } else {
                false
            }
        }
    }

    let timer = Instant::now();
    // Debug
    debug!("Starting pathfind...");
    //*p == goal
    let path = astar(&start, |p| p.successors(&map), |p| p.distance(&goal), |p| smart_goal(&goal, &p, &start));
    debug!("Took {} ms to pathfind.", timer.elapsed().as_millis());

    path
}