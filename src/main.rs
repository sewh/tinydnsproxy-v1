extern crate curl;
extern crate env_logger;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rand;
extern crate regex;
extern crate rustls;
extern crate serde;
extern crate toml;
extern crate webpki;
extern crate webpki_roots;

mod block_list;
mod config;
mod dns_message;
mod error;
mod listener;
mod tls_connection;
mod tls_message;

use block_list::{BlockListFormat, BlockLists};
use config::Config;
use listener::Listener;
use std::env;
use std::process::exit;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

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
            error!("Error loading config: {}", e);
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
                error!("Unknown block list format: {}", entry.format);
                exit(1);
            }
        };

        if entry.list_type == "file" {
            let path = match &entry.path {
                Some(p) => p,
                None => {
                    warn!("There is a file block list entry without a path!");
                    continue;
                }
            };
            if block_lists.add_file(&path, &format).is_err() {
                warn!("Couldn't add file {}", path);
                continue;
            }
        } else if entry.list_type == "http" {
            let url = match &entry.url {
                Some(u) => u,
                None => {
                    warn!("There is a http block list entry without a URL!");
                    continue;
                }
            };
            if block_lists.add_http(&url, &format).is_err() {
                warn!("Couldn't add HTTP URL {}", url);
                continue;
            }
        }
    }

    // Use the config to create a listener
    let mut listener = Listener::from_config(&config);

    // Set blocks lists
    listener.set_blocklists(block_lists);

    // Start up auto update thread
    listener.start_reload_thread();

    // Begin listening and serving
    info!(
        "Starting listener on UDP {}:{}",
        config.bind.host, config.bind.port
    );
    let res = listener.listen_and_serve();
    match res {
        Ok(_) => info!("Done with no errors"),
        Err(e) => error!("Failed: {}", e),
    };

    exit(0);
}
