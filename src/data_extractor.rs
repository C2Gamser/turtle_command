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
use crate::chunks::BlockData;

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

pub struct ModelLoader {
    asset_loader: AssetPack
}

impl ModelLoader {
    pub fn new(root_path: PathBuf) -> Self {
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
    pub fn get_model_props(&self, block_data: &BlockData) -> Option<ModelProperties> {
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

    pub fn get_model_render(&self, model_props: ModelProperties) {
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