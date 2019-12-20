use std::error;
use std::fmt;
use std::io::{Cursor, Read, Write};
use std::net::TcpStream;

use crate::config::Config;
use byteorder::{NetworkEndian, ReadBytesExt};
use native_tls::TlsConnector;
use rand::seq::SliceRandom;

type Result<T> = std::result::Result<T, TlsError>;

#[derive(Debug, Clone)]
pub enum TlsErrorKind {
    NoAvailableServers,
    NoInitialize,
    ConnectFailed,
    TlsHandshakeFailed,
    TlsWriteError,
    TlsReadError,
    BadSizeReturned,
}

#[derive(Debug, Clone)]
pub struct TlsError {
    why: TlsErrorKind,
    server: Option<String>,
}

impl TlsError {
    pub fn new(why: TlsErrorKind) -> TlsError {
        TlsError { why, server: None }
    }

    pub fn set_server(&mut self, server: String) {
        self.server = Some(server.clone());
    }
}

impl fmt::Display for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_append = match &self.server {
            Some(s) => format!("Remote server: {}", s),
            None => "".to_string(),
        };
        let msg = match &self.why {
	    TlsErrorKind::NoAvailableServers => format!("There are no DNS-over-TLS servers to choose from in the config. {}", to_append),
	    TlsErrorKind::NoInitialize => format!("Could not initialize TLS context. {}", to_append),
	    TlsErrorKind::ConnectFailed => format!("Could not create TCP connection with remote server. {}", to_append),
	    TlsErrorKind::TlsHandshakeFailed => format!("Could not create TLS connection with remote server. WARNING it may not be trusted by your operating system! Check the 'hostname' parameter in the config and try again. {}", to_append),
	    TlsErrorKind::TlsWriteError => format!("Could not write data into the TLS stream. {}", to_append),
	    TlsErrorKind::TlsReadError => format!("Could not read data from the TLS stream. {}", to_append),
	    TlsErrorKind::BadSizeReturned => format!("DNS message size is too large or it's malformed. {}", to_append),
	};
        write!(f, "{}", msg)
    }
}

impl error::Error for TlsError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub fn relay_message(msg: &[u8], conf: &Config) -> Result<Vec<u8>> {
    // Choose a random upstream DNS resolver to connect to
    let server = match conf.dns_server.choose(&mut rand::thread_rng()) {
        Some(s) => s,
        None => {
            let e = TlsError::new(TlsErrorKind::NoAvailableServers);
            return Err(e);
        }
    };
    let conn_string = format!("{}:{}", server.ip_address, server.port);

    // Create a new TLS connector and TCP stream and glue them together
    let connector = match TlsConnector::new() {
        Ok(c) => c,
        Err(_) => {
            let e = TlsError::new(TlsErrorKind::NoInitialize);
            return Err(e);
        }
    };

    let stream = match TcpStream::connect(conn_string) {
        Ok(s) => s,
        Err(_) => {
            let mut e = TlsError::new(TlsErrorKind::ConnectFailed);
            e.set_server(format!("{}:{}", server.ip_address, server.port));
            return Err(e);
        }
    };

    let mut tls = match connector.connect(server.hostname.as_str(), stream) {
        Ok(t) => t,
        Err(_) => {
            let mut e = TlsError::new(TlsErrorKind::TlsHandshakeFailed);
            e.set_server(format!("{}:{}", server.ip_address, server.port));
            return Err(e);
        }
    };

    // Write the serialized DNS request into the stream
    match tls.write_all(msg) {
        Ok(_) => (),
        Err(_) => {
            let mut e = TlsError::new(TlsErrorKind::TlsWriteError);
            e.set_server(format!("{}:{}", server.ip_address, server.port));
            return Err(e);
        }
    };

    // Read back the 2 byte length field for the answer and deserialize it to
    // a number we can use
    let mut size_buff = vec![0; 2];
    match tls.read_exact(&mut size_buff) {
        Ok(_) => (),
        Err(_) => {
            let e = TlsError::new(TlsErrorKind::TlsReadError);
            return Err(e);
        }
    };
    let mut size_curs = Cursor::new(size_buff);
    let size = match size_curs.read_u16::<NetworkEndian>() {
        Ok(s) => s as usize,
        Err(_) => {
            let e = TlsError::new(TlsErrorKind::TlsReadError);
            return Err(e);
        }
    };

    // Make sure the answer isn't too large
    if size > 8192 {
        let e = TlsError::new(TlsErrorKind::BadSizeReturned);
        return Err(e);
    }

    // Create a buffer for the full answer and fill it
    let mut response = vec![0; size];
    match tls.read_exact(&mut response) {
        Ok(_) => (),
        Err(_) => {
            let e = TlsError::new(TlsErrorKind::TlsReadError);
            return Err(e);
        }
    };

    // Return the response buffer to the caller
    Ok(response)
}
