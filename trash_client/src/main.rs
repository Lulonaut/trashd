use std::env;
use std::io::Write;
use std::net::{Shutdown, TcpStream};
use std::path::PathBuf;
use std::process::exit;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("Usage: {} TARGETS...", args.get(0).unwrap());
    }

    let mut stream = match TcpStream::connect("127.0.0.1:7213") {
        Ok(stream) => stream,
        Err(_) => {
            eprintln!("Could not connect to daemon. Is it running?");
            exit(1);
        }
    };

    //remove the program name
    args.remove(0);
    for arg in args {
        let path = PathBuf::from(arg);
        match path.canonicalize() {
            Ok(full_path) => {
                let mut path_string = full_path.to_str().unwrap().to_string();
                path_string.push('\n');

                if stream.write_all(path_string.as_bytes()).is_err() {
                    eprintln!("Could not send message to daemon");
                }
            }
            Err(_) => eprintln!("Could not parse path: {}", path.display()),
        }
    }
    stream.shutdown(Shutdown::Both).unwrap();
}
