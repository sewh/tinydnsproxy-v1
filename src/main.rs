extern crate native_tls;
extern crate rand;
extern crate serde;
extern crate toml;

mod config;
mod listener;
mod message;
mod tls_connection;

use config::Config;
use listener::Listener;
use std::env;
use std::process::exit;

fn main() {
    // Validate arguments and collect them into a vector
    let args_iterator = env::args();
    if args_iterator.len() != 2 {
        println!("Usage: tinydnsproxy config.toml");
        exit(0);
    }

    let mut args = Vec::new();
    for arg in args_iterator {
        args.push(arg);
    }

    let config_path = args[1].clone();

    // Load in the config from the provided path
    let config = match Config::from_toml(config_path) {
        Ok(c) => c,
        Err(e) => {
            println!("Error loading config: {}", e);
            exit(1);
        }
    };

    // Use the config to create a listener
    let listener = Listener::from_config(&config);

    // Begin listening and serving
    println!(
        "Starting listener on UDP {}:{}",
        config.bind.host, config.bind.port
    );
    let res = listener.listen_and_serve();
    match res {
        Ok(_) => println!("Done with no errors"),
        Err(e) => println!("Failed: {}", e),
    }

    exit(0);
}
