use file_crawler::prelude::*;
use lz4_flex::block;
use minecraft_assets::api::AssetPack;
use minecraft_assets::schemas::BlockStates::{Multipart, Variants};
use minecraft_assets::schemas::Model;
use minecraft_assets::schemas::blockstates::{ModelProperties, Variant};
use minecraft_assets::schemas::blockstates::multipart::Condition;
use minecraft_assets::schemas::blockstates::multipart::WhenClause::{Or, Single};
use rocket::form::validate::Len;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::PathBuf;
use zipcrawl::ZipManager;
use regex::regex;
use std::io;
use schematic_mesher::{
    BlockPosition, BoundingBox, InputBlock, Mesh, Mesher, MesherConfig, ResourcePack, TintProvider, export_glb, load_resource_pack,
};
use crate::chunks::{BlockData, Chunk};
use crate::coordinates::Coordinate;

pub struct MeshGenerator {
    pub mesher: Mesher
}

impl MeshGenerator {
    pub fn new(resource_path: PathBuf) -> Self {
        let pack = load_resource_pack(resource_path).unwrap();

        let config = MesherConfig {
            cull_hidden_faces: true,      // Remove faces between adjacent blocks
            cull_occluded_blocks: true,   // Skip blocks with all 6 neighbors opaque
            greedy_meshing: false,        // Merge coplanar faces into larger quads
            atlas_max_size: 4096,         // Max texture atlas dimension
            atlas_padding: 1,             // Padding between atlas textures
            include_air: false,           // Skip air blocks
            ambient_occlusion: false,      // Enable AO
            ao_intensity: 1.0,            // AO darkness (0.0-1.0)
            enable_block_light: true,
            enable_particles: false,
            enable_sky_light: true,
            sky_light_level: 5,
            pre_built_atlas: None,
            tint_provider: TintProvider::for_biome("plains"),
        };

        let mesher = Mesher::with_config(pack, config);
        MeshGenerator { mesher }
    }

    // Outputs json as shown here: https://github.com/Schem-at/Schematic-Mesher
    // Useful: https://docs.rs/serde_json/latest/serde_json/
    // Maybe refactor to use the dump_chunk function
    pub fn dump_all_chunks(&self, world_data_path: PathBuf) -> (BoundingBox, Vec<(BlockPosition, InputBlock)>) {
        let files = fs::read_dir(&world_data_path).unwrap();
        let mut chunks: Vec<Chunk> = vec![];

        for dir_entry in files {
            let Ok(dir_entry) = dir_entry else {
                continue;
            };
            let file_name = dir_entry.file_name().to_string_lossy().to_string();

            if file_name == "whitelist" {
                continue;
            }

            let name_split: Vec<&str> = file_name.split("_").collect();
            let x = name_split[0].parse().unwrap();
            let y = name_split[1].parse().unwrap();
            let z = name_split[2].parse().unwrap();
            let new_chunk = Chunk::load(&world_data_path, &Coordinate { x, y, z }).unwrap();

            chunks.push(new_chunk);
        }


        let mut blocks: Vec<(BlockPosition, InputBlock)> = vec![];
        let mut min_bound: [f32; 3] = [f32::MAX, f32::MAX, f32::MAX];
        let mut max_bound: [f32; 3] = [f32::MIN, f32::MIN, f32::MIN];

        for chunk in chunks {
            let c = chunk.coordinates;
            let wc = [c.x as i32, c.y as i32, c.z as i32];
            for row in chunk.block_data.iter().enumerate() {
                for column in row.1.iter().enumerate() {
                    for block in column.1.iter().enumerate() {
                        let name = &block.1.name;
                        // Skip air
                        if name == "minecraft:air" {
                            continue;
                        }
                        // Converts local block coords to world coords
                        let position = [wc[0]*16+row.0 as i32, wc[1]*16+column.0 as i32, wc[2]*16+block.0 as i32];
                        min_bound[0] = min_bound[0].min(position[0] as f32);
                        min_bound[1] = min_bound[1].min(position[1] as f32);
                        min_bound[2] = min_bound[2].min(position[2] as f32);

                        max_bound[0] = max_bound[0].max(position[0] as f32);
                        max_bound[1] = max_bound[1].max(position[1] as f32);
                        max_bound[2] = max_bound[2].max(position[2] as f32);

                        let mut input_block = InputBlock::new(name);
                        // Apply properties
                        for (key, value) in  block.1.states.iter().map(|f|{(f.0.to_string(), f.1.to_string())}) {
                            input_block.properties.insert(key, value);
                        };

                        blocks.push((BlockPosition::new(position[0],position[1], position[2]), input_block));
                    }
                }
            }
        }

        let bounds = BoundingBox::new(min_bound, max_bound);

        return (bounds, blocks);
    }

    pub fn dump_chunk(&self, chunk: Chunk) -> (BoundingBox, Vec<(BlockPosition, InputBlock)>) {
        let mut blocks: Vec<(BlockPosition, InputBlock)> = vec![];

        let c = chunk.coordinates;
        let wc = [c.x as i32 * 16, c.y as i32 * 16, c.z as i32 * 16];
        for row in chunk.block_data.iter().enumerate() {
            for column in row.1.iter().enumerate() {
                for block in column.1.iter().enumerate() {
                    let name = &block.1.name;
                    // Skip air
                    if name == "minecraft:air" {
                        continue;
                    }
                    // Converts local block coords to world coords
                    let position = [wc[0]+row.0 as i32, wc[1]+column.0 as i32, wc[2]+block.0 as i32];

                    let mut input_block = InputBlock::new(name);
                    // Apply properties
                    for (key, value) in  block.1.states.iter().map(|f|{(f.0.to_string(), f.1.to_string())}) {
                        input_block.properties.insert(key, value);
                    };

                    blocks.push((BlockPosition::new(position[0],position[1], position[2]), input_block));
                }
            }
        }

        let bounds = BoundingBox::new(wc.map(|f|f as f32), wc.map(|f|f as f32 + 16.0));

        return (bounds, blocks);
    }

    // Returns a byte vec of a glb to be exported directly
    pub fn mesh_chunk(&self, chunk: Chunk) -> Vec<u8> {
        let (bounding_box, block_data) = self.dump_chunk(chunk);

        let output = self.mesher.mesh_blocks(
            block_data.iter().map(|(pos, block)| (*pos, block)),
            bounding_box,
        ).unwrap();

        let glb_bytes = export_glb(&output).unwrap();
        glb_bytes
    }

    // Also normalizes the bounding box to start at 0, 0
    pub fn mesh_all_chunks(&self, world_data_path: PathBuf) -> Vec<u8> {
        let (bounding_box, block_data) = self.dump_all_chunks(world_data_path);

        let offset = bounding_box.min;
        let new_min: [f32; 3] = [bounding_box.min[0]-offset[0],bounding_box.min[1]-offset[1], bounding_box.min[2]-offset[2]];
        let new_max: [f32; 3] = [bounding_box.max[0]-offset[0],bounding_box.max[1]-offset[1], bounding_box.max[2]-offset[2]];
        let normalized_bounds = BoundingBox::new(new_min, new_max);

        let block_data: Vec<(BlockPosition, InputBlock)> = block_data.iter().map(|(f, a)| {
            (BlockPosition::new(f.x-offset[0] as i32, f.y-offset[1] as i32, f.z-offset[2] as i32), a.clone())
        }).collect();

        let output = self.mesher.mesh_blocks(
            block_data.iter().map(|(pos, block)| (*pos, block)),
            normalized_bounds,
        ).unwrap();

        let glb_bytes = export_glb(&output).unwrap();
        glb_bytes
    }
}

pub struct MCDataCrawler {
    start_path: PathBuf,
    output_path: PathBuf
}

impl MCDataCrawler {
    pub fn new(start_path: PathBuf, output_path: PathBuf) -> Self {
        MCDataCrawler { start_path,  output_path }
    }

    pub fn extract_data(&self) {
        let _count= Crawler::new()
            .start_dir(&self.start_path)
            .file_regex(r"^.*\.jar$")
            .run(|_, path: PathBuf| {
                if !path.to_string_lossy().contains("processedMods") {
                    let mut file = File::open(&path).unwrap();

                    let mut zip_crawler = ZipManager::from_reader(&mut file).unwrap();

                    let zip_contents = zip_crawler.entries().unwrap();

                    for entry in zip_contents.iter() {
                        // Match all asset json and png files for blocks
                        if regex!(r"^assets/").is_match(&entry.name) && // Starts with assets/
                        regex!(r"/(block|blockstates)/").is_match(&entry.name) && // contains any of these surrounded by /
                        regex!(r"\.(json|png)$").is_match(&entry.name) // Ends with .json or .png
                            {
                            // let trimmed_destination = regex!(r"^assets/").replace(&entry.name, "");
                            // let trimmed_destination = regex!(r"/").replace_all(&trimmed_destination, "\\").to_string();
                            let mut final_destination = self.output_path.to_string_lossy().to_string();
                            final_destination.push_str(&"/".to_string());
                            final_destination.push_str(&entry.name.to_string());

                            let final_destination = PathBuf::from(final_destination);

                            let mut final_destination_dir = final_destination.clone();
                            final_destination_dir.pop();

                            if fs::exists(&final_destination).unwrap() {
                                continue
                            }

                            std::fs::create_dir_all(final_destination_dir).unwrap();
                            let mut inner_file = File::create(PathBuf::from(&final_destination)).unwrap();

                            let _ = zip_crawler.stream_file(&entry.name, |reader| {
                                io::copy(reader, &mut inner_file).expect("Failed to copy content to file");
                                println!("Extracted to {:?}", final_destination);
                                Ok(())
                            });
                        }
                    }
                }
                //placeholder error type for now
                Ok::<(), std::io::Error>(())
            });
    }
}

struct ModelLoader {
    asset_loader: AssetPack
}

impl ModelLoader {
    fn new(root_path: PathBuf) -> Self {
        ModelLoader { asset_loader: minecraft_assets::api::AssetPack::at_path(root_path) }
    }

    fn condition_match_blockstate(&self, block_data: &BlockData, condition: Condition) -> bool {
        for (key, value) in condition.and {
            let my_val = minecraft_assets::schemas::blockstates::multipart::StateValue::String(block_data.states[&key].to_string());
            if my_val != value {
                return false
            }
        }
        return true
    }

    // Note: ALWAYS returns the first applicable model instead of choosing randomly when there are multiple to choose from
    fn get_model_props(&self, block_data: &BlockData) -> Option<ModelProperties> {
        let variants = self.asset_loader.load_blockstates(&block_data.name);
        // Verify that we have block states
        let Ok(block_states) = variants else {
            return None
        };

        // Deal with multipart and variants separately
        match block_states {
            Multipart { cases } => {
                // For each case that applies
                for case in cases {
                    let when = case.when;

                    // Ignore nonexistent cases
                    let Some(when) = when else {
                        continue
                    };

                    // Deal with single and or cases separately
                    match &when {
                        Single(condition) => {
                            let matches = self.condition_match_blockstate(&block_data, condition.clone());
                            if matches == true {
                                // Note: ALWAYS returns the first model (instead of choosing randomly given multiple)
                                return Some(case.apply.models()[0].clone())
                            }
                        },
                        Or{ or } => {
                            for condition in or {
                                let matches = self.condition_match_blockstate(&block_data, condition.clone());
                                if matches == true {
                                    // Note: ALWAYS returns the first model (instead of choosing randomly given multiple)
                                    return Some(case.apply.models()[0].clone())
                                }
                            }
                        },
                    }
                }
            }
            Variants { variants } => {
                for variant in variants {
                    let mut matches = 0;
                    for (state, value) in &block_data.states {
                        if variant.0.contains(&format!("{}={}",state,value.to_string())) {
                            matches += 1
                        }
                    }
                    // Count the number of = signs to tell how many values it has
                    // Note that this works with zero tag blockstates e.g. yellow_concrete_powder where the tag is simply ""
                    if matches == variant.0.chars().filter(|c| *c == '=').count() {
                        return Some(variant.1.models()[0].clone())
                    }
                }
            }
        }
        return None
    }

    fn get_model_render(&self, model_props: ModelProperties) {
        let model_list = self.asset_loader.load_block_model_recursive(&model_props.model).unwrap();
        for model in model_list {

        }
    }
}

// Useful resources:
// https://minecraft.wiki/w/Tutorial:Models#Example:_Standing_Torch
// https://minecraft.wiki/w/Blockstates_definition
// VERY helpful for serde: https://serde.rs/enum-representations.html
// https://github.com/serde-rs/serde/issues/1560