use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use crate::block_list::BlockLists;
use crate::config::Config;
use crate::dns_message;
use crate::tls_connection;
use crate::tls_message;

#[derive(Debug)]
pub struct Listener {
    config: Config,
    block_lists: Arc<RwLock<Option<BlockLists>>>,
    shut_tx: mpsc::Sender<bool>,
    shut_rx: mpsc::Receiver<bool>,
}

impl Listener {
    pub fn from_config(config: &Config) -> Listener {
        let (tx, rx) = mpsc::channel::<bool>();
        let c = config.clone();
        let block_lists = Arc::new(RwLock::new(None));
        Listener {
            config: c,
            block_lists: block_lists,
            shut_tx: tx,
            shut_rx: rx,
        }
    }

    pub fn set_blocklists(&mut self, block_lists: BlockLists) {
        let block_lists = Arc::new(RwLock::new(Some(block_lists)));
        self.block_lists = block_lists;
    }

    pub fn _shutdown(&self) {
        match self.shut_tx.send(true) {
            Ok(_) => (),
            Err(_) => (),
        }
    }

    pub fn listen_and_serve(&self) -> io::Result<()> {
        let conn_string = format!("{}:{}", self.config.bind.host, self.config.bind.port);
        let socket = UdpSocket::bind(conn_string)?;
        let mut buffer = vec![0; 8192];
        loop {
            // First, check if we are being told to shutdown
            match self.shut_rx.try_recv() {
                Ok(_) => break,
                Err(_) => (),
            };

            // Await a DNS request (TODO make this non-blocking)
            let (amt, src) = match socket.recv_from(buffer.as_mut_slice()) {
                Ok(res) => res,
                Err(e) => {
                    println!("Error: {}", e);
                    continue;
                }
            };

            // Copy buffer into correctly sized buffed so another thread
            // can safely own its own copy of the message
            let mut local_buff = Vec::with_capacity(amt);
            for i in 0..amt {
                local_buff.push(buffer[i]);
            }

            // Clone the socket and pass everything to handle request
            let s = match socket.try_clone() {
                Ok(s) => s,
                Err(_) => continue,
            };
            self.handle_request(local_buff, s, src);

            // Zero out the space we used in the global buffer
            for i in 0..amt {
                buffer[i] = 0x0;
            }
        }

        Ok(())
    }

    fn handle_request(&self, msg: Vec<u8>, socket: UdpSocket, src: SocketAddr) {
        let c = self.config.clone();
        let block_lists = self.block_lists.clone();

        // Spin up a new thread to handle this from now on
        thread::spawn(move || {
            // Serialize the raw DNS query into one compatible with DNS-over-TLS
            let serialized = match tls_message::serialize(&msg) {
                Ok(m) => m,
                Err(e) => {
                    println!("Error serializing buffer: {}", e);
                    return;
                }
            };

            // Check to see if the domain is in the block list
            let mut should_block = false;
            match dns_message::hostname_from_bytes(&msg) {
                Ok(hostname) => {
                    // Get a read-only handle to the block lists and check them. If we
		    // can't get the handle because the block lists are being updated just
		    // nope out and let the request pass unblocked
		    match block_lists.try_read() {
			Ok(optional) => {
			    match &*optional {
				Some(bl) => {
				    should_block = bl.is_blocked(&hostname);
				},
				None => ()
			    }
			},
			Err(_) => ()
		    }
                }
                Err(_) => (), // TODO add some logging or something
            }

            let res = match should_block {
                true => {
                    match dns_message::create_nxdomain(&msg) {
                        Ok(r) => r,
                        Err(_) => {
                            // TODO add some logging here
                            return;
                        }
                    }
                }
                false => match tls_connection::relay_message(serialized.as_slice(), &c) {
                    Ok(res) => res,
                    Err(e) => {
                        println!("TLS Error: {}", e);
                        return;
                    }
                },
            };

            // Send the response back to the client
            match socket.send_to(res.as_slice(), &src) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error sending response: {}", e);
                    return;
                }
            }
        });
    }
}
