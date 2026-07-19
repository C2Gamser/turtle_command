use file_crawler::prelude::*;
use rocket::serde::json::Value;
use ::serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::iter::Map;
use std::path::PathBuf;
use zipcrawl::ZipManager;
use regex::regex;
use std::io::{self, Read};
use rocket::{serde};
use serde::json::serde_json;
use minecraft_assets::api::AssetPack;
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
                            let trimmed_destination = regex!(r"^assets/").replace(&entry.name, "");
                            // let trimmed_destination = regex!(r"/").replace_all(&trimmed_destination, "\\").to_string();
                            let mut final_destination = self.output_path.to_string_lossy().to_string();
                            final_destination.push_str(&"/".to_string());
                            final_destination.push_str(&trimmed_destination.to_string());

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

#[derive(Debug, serde::Deserialize)]
enum VariantEnum {
    #[serde(rename = "")]
    one(Vec<Model>),
    #[serde(untagged)]
    two(HashMap<String, Model>),
}

#[derive(Debug, serde::Deserialize)]
struct Model {
    model: String,
    x: Option<i32>,
    y: Option<i32>,
    z: Option<i32>,
    uvlock: Option<bool>,
    weight: Option<i32>,
}

#[derive(Debug, serde::Deserialize)]
struct BlockStates {
    variants: VariantEnum
}

pub struct MC3DModel {
    extracted_data_path: PathBuf,
}

impl MC3DModel {
    pub fn new(extracted_data_path: PathBuf) -> Self {
        MC3DModel { extracted_data_path }
    }

    pub fn load_obj(&self, block_name: String) {
        let namespace = regex!(r"^[^:]+").find(&block_name).unwrap().as_str();
        let name = &regex!(r"^[^:]+:(.+)$").captures(&block_name).unwrap()[1];

        println!("{}",self.extracted_data_path.to_str().unwrap().to_owned()+"/"+namespace+"/blockstates/"+name+".json");

        let blockstates_data = fs::read_to_string(self.extracted_data_path.to_str().unwrap().to_owned()+"/"+namespace+"/blockstates/"+name+".json").unwrap();


        let blockstates_deserialized  = serde::json::from_str::<Value>(&blockstates_data).unwrap();


        dbg!(blockstates_deserialized);
    }
}

