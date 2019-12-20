use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::thread;

use crate::config::Config;
use crate::message;
use crate::tls_connection;

#[derive(Debug)]
pub struct Listener {
    config: Config,
    shut_tx: mpsc::Sender<bool>,
    shut_rx: mpsc::Receiver<bool>,
}

impl Listener {
    pub fn from_config(config: &Config) -> Listener {
        let (tx, rx) = mpsc::channel::<bool>();
        let c = config.clone();
        Listener {
            config: c,
            shut_tx: tx,
            shut_rx: rx,
        }
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

        // Spin up a new thread to handle this from now on
        thread::spawn(move || {
            // Serialize the raw DNS query into one compatible with DNS-over-TLS
            let serialized = match message::serialize(&msg) {
                Ok(m) => m,
                Err(e) => {
                    println!("Error serializing buffer: {}", e);
                    return;
                }
            };

            // Spin up a TLS connection to a provider, fire the query at them, and get a
            // response
            let res = match tls_connection::relay_message(serialized.as_slice(), &c) {
                Ok(res) => res,
                Err(e) => {
                    println!("TLS Error: {}", e);
                    return;
                }
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
