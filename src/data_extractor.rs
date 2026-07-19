use file_crawler::prelude::*;
use std::fs::{self, File};
use std::path::PathBuf;
use zipcrawl::ZipManager;
use regex::regex;
use std::io;

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

