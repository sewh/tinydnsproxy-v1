use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{atomic, Arc, RwLock};
use std::time::{Duration, Instant};
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
    should_stop: Arc<atomic::AtomicBool>,
    reload_thread: Option<thread::JoinHandle<()>>,
}

impl Listener {
    pub fn from_config(config: &Config) -> Listener {
        let c = config.clone();
        let block_lists = Arc::new(RwLock::new(None));
        let l = Listener {
            config: c,
            block_lists: block_lists,
	    should_stop: Arc::new(atomic::AtomicBool::new(false)),
	    reload_thread: None,
        };

	l
    }

    pub fn start_reload_thread(&mut self) {
	// Check to make sure we should run a reload thread
	let block_lists = match &self.config.block_lists {
	    Some(bl) => bl,
	    None => return,
	};
	let refresh_after = match block_lists.refresh_after {
	    Some(ra) => ra,
	    None => return,
	};

	if refresh_after == 0 {
	    return;
	}

	let should_stop = self.should_stop.clone();
	let interval_seconds = refresh_after * 60;
	let block_lists = Arc::clone(&self.block_lists);

	info!("Will reload block lists every {} minutes", refresh_after);

	// Okay, we have a proper refresh after
	let t = thread::spawn(move || {
	    let mut current_instant = Instant::now();
	    loop {
		// Check if we should stop this thread
		if should_stop.load(atomic::Ordering::Relaxed) {
		    break;
		}

		if current_instant.elapsed().as_secs() > interval_seconds {
		    let mut bl_option = match block_lists.write() {
			Ok(bl_option) => bl_option,
			Err(_) => {
			    current_instant = Instant::now();
			    continue
			},
		    };

		    let bl = match &mut *bl_option {
			Some(bl) => bl,
			None => {
			    std::mem::drop(bl_option);
			    current_instant = Instant::now();
			    continue;
			}
		    };

		    // TODO add some proper error handling stuff here
		    match bl.reload_lists() {
			Ok(_) => {
			    info!("Reloaded block lists successfully")
			},
			Err(e) => {
			    warn!("Couldn't refresh block lists: {}", e)
			}
		    }

		    current_instant = Instant::now();

		    std::mem::drop(bl_option);

		}
		thread::sleep(Duration::from_secs(1));
	    }
	    info!("Stopping block list update thread");
	});

	self.reload_thread = Some(t);
    }

    pub fn set_blocklists(&mut self, block_lists: BlockLists) {
        let block_lists = Arc::new(RwLock::new(Some(block_lists)));
        self.block_lists = block_lists;
    }

    pub fn _shutdown(&mut self) {
	self.should_stop.store(true, atomic::Ordering::Relaxed);

	if let Some(t) = self.reload_thread.take() {
	    match t.join() {
		_ => (),
	    }
	}
    }

    pub fn listen_and_serve(&self) -> io::Result<()> {
        let conn_string = format!("{}:{}", self.config.bind.host, self.config.bind.port);
        let socket = UdpSocket::bind(conn_string)?;
	socket.set_read_timeout(Some(Duration::from_secs(1)))?;
        let mut buffer = vec![0; 8192];
        loop {

	    if self.should_stop.load(atomic::Ordering::Relaxed) {
		break;
	    }

            // Await a DNS request. If nothing then loop around again so we get an
	    // opportunity to check if we should shutdown
            let (amt, src) = match socket.recv_from(buffer.as_mut_slice()) {
                Ok(res) => res,
		Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
		    continue;
		},
		Err(e) if e.kind() == io::ErrorKind::TimedOut => {
		    continue;
		},
                Err(e) => {
                    warn!("Error: {}", e);
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
                    warn!("Error serializing buffer: {}", e);
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
				    debug!("Blocking domain: {}", hostname);
				    should_block = bl.is_blocked(&hostname);
				},
				None => debug!("Not blocking domain: {}", hostname),
			    }
			},
			Err(_) => (),
		    }
                }
                Err(_) => {
		    warn!("Could not extract hostname from DNS(?) message!");
		},
            }

            let res = match should_block {
                true => {
                    match dns_message::create_nxdomain(&msg) {
                        Ok(r) => r,
                        Err(_) => {
                            warn!("Could not create a NX domain message!");
                            return;
                        }
                    }
                }
                false => match tls_connection::relay_message(serialized.as_slice(), &c) {
                    Ok(res) => res,
                    Err(e) => {
                        warn!("TLS Error: {}", e);
                        return;
                    }
                },
            };

            // Send the response back to the client
            match socket.send_to(res.as_slice(), &src) {
                Ok(_) => (),
                Err(e) => {
                    warn!("Error sending response: {}", e);
                    return;
                }
            }
        });
    }
}
