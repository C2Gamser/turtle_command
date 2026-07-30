use file_crawler::prelude::*;
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Mutex;
use zipcrawl::ZipManager;
use regex::regex;
use std::io;
use schematic_mesher::{
    BlockPosition, BoundingBox, InputBlock, Mesher, MesherConfig, TintProvider, export_glb, load_resource_pack,
};
use crate::chunks::Chunk;

pub struct MeshGenerator {
    pub mesher: Mutex<Mesher>
}

impl MeshGenerator {
    pub async fn new(resource_path: PathBuf) -> Self {
        let pack = load_resource_pack(resource_path).unwrap();

        let config = MesherConfig {
            cull_hidden_faces: true,      // Remove faces between adjacent blocks
            cull_occluded_blocks: true,   // Skip blocks with all 6 neighbors opaque
            greedy_meshing: false,        // Merge coplanar faces into larger quads
            atlas_max_size: 4096,         // Max texture atlas dimension
            atlas_padding: 1,             // Padding between atlas textures
            include_air: false,           // Skip air blocks
            ambient_occlusion: true,      // Enable AO
            ao_intensity: 1.0,            // AO darkness (0.0-1.0)
            enable_block_light: false,
            enable_particles: false,
            enable_sky_light: false,
            sky_light_level: 5,
            pre_built_atlas: None,
            tint_provider: TintProvider::for_biome("plains"),
        };

        let mesher = Mesher::with_config(pack, config);

        MeshGenerator { mesher: Mutex::new(mesher) }
    }

    // Outputs json as shown here: https://github.com/Schem-at/Schematic-Mesher
    // Useful: https://docs.rs/serde_json/latest/serde_json/

    pub fn dump_chunk(&self, chunk: Chunk) -> (BoundingBox, Vec<(BlockPosition, InputBlock)>) {
        let mut blocks: Vec<(BlockPosition, InputBlock)> = vec![];

        for row in chunk.block_data.iter().enumerate() {
            for column in row.1.iter().enumerate() {
                for block in column.1.iter().enumerate() {
                    let name = &block.1.name;
                    // Skip air + grass as grass is annoying
                    if name == "minecraft:air" || name == "minecraft:short_grass" {
                        continue;
                    }
                    // Places the blocks locally within the chunk
                    let position = [row.0 as i32, column.0 as i32, block.0 as i32];

                    let mut input_block = InputBlock::new(name);
                    // Apply properties
                    for (key, value) in  block.1.states.iter().map(|f|{(f.0.to_string(), f.1.to_string())}) {
                        input_block.properties.insert(key, value);
                    };

                    blocks.push((BlockPosition::new(position[0],position[1], position[2]), input_block));
                }
            }
        }

        let bounds = BoundingBox::new([0.0, 0.0, 0.0], [16.0, 16.0, 16.0]);
        return (bounds, blocks);
    }

    // Returns a byte vec of a glb to be exported along with the world coordinates of that chunk (chunk coords * 16)
    pub fn mesh_chunk(&self, chunk: Chunk) -> Option<Vec<u8>> {
        let (bounding_box, block_data) = self.dump_chunk(chunk);

        let output = self.mesher.lock().unwrap().mesh_blocks(
            block_data.iter().map(|(pos, block)|(*pos, block)),
            bounding_box,
        ).unwrap();

        let glb_bytes = export_glb(&output);

        let Ok(glb_bytes) = glb_bytes else {
            return None
        };

        return Some(glb_bytes)
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

// Useful resources:
// https://minecraft.wiki/w/Tutorial:Models#Example:_Standing_Torch
// https://minecraft.wiki/w/Blockstates_definition
// VERY helpful for serde: https://serde.rs/enum-representations.html
// https://github.com/serde-rs/serde/issues/1560