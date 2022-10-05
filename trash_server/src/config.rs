use std::{fs, process};
use std::path::PathBuf;

pub struct Config {
    delete_after: i32,
}

impl Config {
    pub fn from_file(path: PathBuf) -> Config {
        match fs::read_to_string(&path) {
            Ok(contents) => {
                dbg!(contents);

                Config{ delete_after:1, }
            }
            Err(_) => {
                eprintln!("Error while reading config file: {}", path.display());
                process::exit(1);
            }
        }
    }
    
    pub fn get_default_config_string() -> String {
        let mut default_config = String::new();
        default_config.push_str("delete_after:1");
        default_config
    }
}