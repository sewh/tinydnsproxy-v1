#[macro_use] extern crate lazy_static;
extern crate native_tls;
extern crate rand;
extern crate serde;
extern crate regex;
extern crate toml;

mod block_list;
mod config;
mod dns_message;
mod listener;
mod message;
mod tls_connection;

use block_list::{BlockLists, BlockListFormat};
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

    // Create a block list from the config
    let mut block_lists = BlockLists::new();
    for entry in &config.block_list {
	let format = match entry.format.as_str() {
	    "hosts" => BlockListFormat::Hosts,
	    "one-per-line" => BlockListFormat::OnePerLine,
	    _ => {
		println!("Unknown block list format: {}", entry.format);
		exit(1);
	    }
	};

	if entry.format == "file" {
	    let path = match &entry.path {
		Some(p) => p,
		None => continue
	    };
	    if block_lists.add_file(&path, &format).is_err() {
		continue;
	    }
	}
    }

    // Use the config to create a listener
    let mut listener = Listener::from_config(&config);

    // Set blocks lists
    listener.set_blocklists(block_lists);

    // Begin listening and serving
    println!(
        "Starting listener on UDP {}:{}",
        config.bind.host, config.bind.port
    );
    let res = listener.listen_and_serve();
    match res {
        Ok(_) => println!("Done with no errors"),
        Err(e) => println!("Failed: {}", e),
    };

    exit(0);
}
