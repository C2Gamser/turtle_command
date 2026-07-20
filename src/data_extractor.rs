use file_crawler::prelude::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::PathBuf;
use zipcrawl::ZipManager;
use regex::regex;
use std::io;
use crate::chunks::BlockData;
use rocket::{serde};

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
    ModelList(Vec<ModelData>),
    #[serde(untagged)]
    VariantList(HashMap<String, ModelData>),
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct ModelData {
    pub model: String,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub z: Option<i32>,
    pub uvlock: Option<bool>,
    pub weight: Option<i32>,
}

#[derive(Debug, serde::Deserialize)]
struct VariantStates {
    variants: VariantEnum
}

#[derive(Debug, serde::Deserialize)]
enum WhenEnum {
    OR(Vec<HashMap<String, String>>),
    #[serde(untagged)]
    None(HashMap<String, String>)
}

#[derive(Debug, serde::Deserialize)]
struct Case {
    apply: ModelData,
    when: Option<WhenEnum>
}

#[derive(Debug, serde::Deserialize)]
enum VariantOrMultipart {
    #[serde(rename = "multipart")]
    Multipart(Vec<Case>),
    #[serde(rename = "variants")]
    Variants(VariantEnum)
}

pub struct MC3DModelExtractor {
    extracted_data_path: PathBuf,
}

impl MC3DModelExtractor {
    pub fn new(extracted_data_path: PathBuf) -> Self {
        MC3DModelExtractor { extracted_data_path }
    }

    pub fn parse_blockstates(&self, block_data: BlockData) -> Result<ModelData, String> {
        let namespace_name = &block_data.name.split_once(':').unwrap();
        let namespace = namespace_name.0;
        let name = namespace_name.1;

        // println!("{}",self.extracted_data_path.to_str().unwrap().to_owned()+"/"+namespace+"/blockstates/"+name+".json");

        let blockstates_data = fs::read_to_string(self.extracted_data_path.to_str().unwrap().to_owned()+"/"+namespace+"/blockstates/"+name+".json").unwrap();

        let blockstates_deserialized  = serde::json::from_str::<VariantOrMultipart>(&blockstates_data).unwrap();

        match &blockstates_deserialized {
            // TODO: Finish multipart
            // e.g. acacia_fence
            VariantOrMultipart::Multipart(multipart) => {
                return Err("Multipart models are not yet implemented.".into())
            }

            VariantOrMultipart::Variants(variants) => {
                match variants {
                    // e.g. yellow_concrete_powder
                    // We simplify this greatly by just returning the first model in the list
                    VariantEnum::ModelList(model_list) => {
                        return Ok(model_list[0].clone())
                    }
                    // e.g. yellow_cake
                    VariantEnum::VariantList(variant_list) => {
                        // We are trying to get the block states list to look like "candles=1,lit=false"
                        let mut block_states_combined = "".to_string();
                        let block_state_len = block_data.states.len();

                        for block_state in block_data.states.into_iter().enumerate() {
                            let pos = block_state.0;
                            let block_state = block_state.1;
                            block_states_combined.push_str(&block_state.0);
                            block_states_combined.push_str("=");
                            block_states_combined.push_str(&block_state.1.to_string());
                            // If we are at the last element don't add a comma
                            if pos != block_state_len - 1 {
                                block_states_combined.push_str(",");
                            }
                        }
                        return Ok(variant_list[&block_states_combined].clone())
                    }
                }
            }
        }
    }


}

