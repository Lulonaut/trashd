use std::ffi::OsString;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, thread};
use std::{process, time};

use crate::config::Config;

mod config;

const PORT: i32 = 7213;
const PATH: &str = "trash_folder";

///Handle an incoming connection in form of a TcpStream
///
/// Read all the input from this stream and try adding it as an entry
fn handle_connection(mut stream: TcpStream) {
    // Handle multiple access stream
    let mut buf: Vec<u8> = Vec::new();
    if stream.read_to_end(&mut buf).is_err() {
        eprintln!("Error while reading from stream.");
        return;
    }

    let contents = String::from_utf8_lossy(&buf).to_string();
    contents.lines().for_each(add_entry);
}

///Add an entry in form of a path to the trash files.
///
/// This includes moving the entry on the filesystem to the files folder and creating an entry for it in the info folder
fn add_entry(line: &str) {
    let path = Path::new(line);
    let mut file_name = path.file_name().unwrap().to_os_string();

    let entries = fs::read_dir(format!("{PATH}/files"))
        .unwrap()
        .flatten()
        .map(|d| d.file_name())
        .collect::<Vec<OsString>>();

    let mut new_path = PathBuf::from(format!("{PATH}/files/a.a")).with_file_name(&file_name);
    for existing_file_name in &entries {
        if file_name.eq(existing_file_name) {
            let mut highest_number = -1;
            let mut no_extension = false;
            for name in &entries {
                let path_buf = PathBuf::from(name);
                let extension = path_buf.extension();
                let name_only_buf = path_buf.with_extension("");
                let name_only = name_only_buf.file_name();

                match name_only {
                    Some(name_only) => match extension {
                        Some(extension) => {
                            if let Some(extension) = extension.to_str() {
                                if file_name.eq(name_only) {
                                    match extension.parse::<i32>() {
                                        Ok(number) => {
                                            if number > highest_number {
                                                highest_number = number;
                                            }
                                        }
                                        Err(_) => {
                                            highest_number = 0;
                                        }
                                    }
                                } else if file_name.eq(path_buf.file_name().unwrap()) {
                                    highest_number = 0;
                                }
                            }
                        }
                        None => {
                            if file_name.eq(name_only) {
                                no_extension = true;
                            }
                        }
                    },
                    None => continue,
                }
            }
            if no_extension && highest_number == -1 {
                highest_number = 0;
            }

            if highest_number >= 0 {
                file_name.push(format!(".{}", highest_number + 1));
                new_path.set_file_name(&file_name);
            }
            break;
        }
    }

    // dbg!(&new_path);
    // "move" the file to the new location
    if fs::rename(&path, &new_path).is_err() {
        eprintln!("Could not move file: {}", line);
    }

    let current_time = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let mut info_file_path = PathBuf::from(format!("{PATH}/info"));
    info_file_path.push(&file_name);
    // dbg!(&info_file_path);

    let mut contents = String::with_capacity(256);
    contents.push_str(format!("added:{current_time}\n").as_str());
    contents.push_str(format!("original_path:{}\n", &path.display()).as_str());

    match File::create(&info_file_path) {
        Ok(mut file) => {
            if file.write_all(contents.as_bytes()).is_err() {
                eprintln!(
                    "Failed to write to info file: {}",
                    &info_file_path.display()
                );
            }
        }
        Err(_) => eprintln!("Failed to create info file for: {}", &path.display()),
    }
}

///Ensure all required folders needed for operation exist.
fn init_folders() {
    ensure_folder_exists(PATH);
    ensure_folder_exists(format!("{PATH}/files"));
    ensure_folder_exists(format!("{PATH}/info"));
}

///Check if the specified folder exists. If it doesn't, try to create it.
///
/// If the creation fails, stop the program.
fn ensure_folder_exists<S>(path_string: S)
where
    S: Into<String>,
    S: Display,
{
    eprint!("Ensuring folder exists: {path_string}");
    let path = PathBuf::from(&path_string.to_string());
    if !path.is_dir() && fs::create_dir(path).is_err() {
        eprintln!("\nCould not create folder: {path_string}");
        eprintln!("This will make operating the daemon impossible, stopping.");
        process::exit(1);
    }
    eprintln!(" ok");
}

fn ensure_config_exists() {
    let path = PathBuf::from(format!("{PATH}/config.conf"));
    eprint!("Ensuring config exists at path: {}", &path.display());
    if !path.is_file() {
        match File::create(&path) {
            Ok(mut file) => {
                if file
                    .write_all(Config::get_default_config_string().as_bytes())
                    .is_err()
                {
                    eprintln!(" failed!");
                    eprintln!("Could not write default values to config file.");
                    eprintln!("This will make operating the daemon impossible, stopping.");
                    process::exit(1);
                }
            }
            Err(_) => {
                eprintln!(" failed!");
                eprintln!("Could not create file: {}", &path.display());
                eprintln!("This will make operating the daemon impossible, stopping.");
                process::exit(1);
            }
        }
    }
    eprintln!(" ok");
}

fn main() -> ! {
    eprintln!("Initializing folders");
    init_folders();
    eprintln!("Initializing config");
    ensure_config_exists();
    let config = Config::from_file(PathBuf::from(format!("{PATH}/config.txt")));
    process::exit(0);

    //Check for incoming connections from clients and handle them
    eprintln!("Spawning thread to accept connections from clients");
    thread::Builder::new()
        .name("SocketListener".to_string())
        .spawn(|| {
            let listener = TcpListener::bind(format!("127.0.0.1:{PORT}"))
                .expect(&*format!("Failed to bind on port: {PORT}"));
            eprintln!("Started socket on port {PORT}");

            for incoming_stream in listener.incoming() {
                match incoming_stream {
                    Ok(stream) => handle_connection(stream),
                    Err(_) => {
                        eprintln!("Error with incoming stream");
                        continue;
                    }
                };
            }
        })
        .unwrap();

    //periodic checking if files need to be deleted
    loop {
        //TODO
        thread::sleep(Duration::from_secs(1000));
    }
}
