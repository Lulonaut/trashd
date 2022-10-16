use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, hash, process};

#[derive(Debug)]
pub struct Config {
    pub delete_after: isize,
}

pub trait Conversion {
    fn to_int(&self) -> isize;
}

pub trait GetOrDefault<T, P> {
    fn get_or_default(&self, key: T, default: P) -> P;
}

impl Conversion for String {
    fn to_int(&self) -> isize {
        self.parse::<isize>().unwrap_or(-1)
    }
}

impl<T, P> GetOrDefault<T, P> for HashMap<T, P>
where
    T: hash::Hash,
    T: Eq,
    P: Clone,
{
    fn get_or_default(&self, key: T, default: P) -> P {
        if self.contains_key(&key) {
            return self.get(&key).unwrap().to_owned();
        }
        default
    }
}

pub fn parse_lines(contents: &str, entries: &mut HashMap<String, String>) {
    for line in contents.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match line.split_once(':') {
            Some(split) => {
                let name = split.0;
                let value = split.1;
                entries.insert(name.to_string(), value.to_string());
            }
            None => {
                eprintln!("Could not read line: {line}");
                eprintln!("No \":\" found");
                continue;
            }
        }
    }
}

impl Config {
    pub fn from_file(path: PathBuf) -> Config {
        match fs::read_to_string(&path) {
            Ok(contents) => {
                let mut entries: HashMap<String, String> = HashMap::new();
                parse_lines(&contents, &mut entries);

                Config {
                    delete_after: entries
                        .get_or_default("delete_after".to_string(), "1".to_string())
                        .to_int(),
                }
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
