use std::io::{Cursor, Read, Write};
use std::net::TcpStream;

use crate::config::Config;
use crate::error::DoTError;
use byteorder::{NetworkEndian, ReadBytesExt};
use native_tls::TlsConnector;
use rand::seq::SliceRandom;

type Result<T> = std::result::Result<T, DoTError>;

pub fn relay_message(msg: &[u8], conf: &Config) -> Result<Vec<u8>> {
    // Choose a random upstream DNS resolver to connect to
    let server = match conf.dns_server.choose(&mut rand::thread_rng()) {
        Some(s) => s,
        None => {
            let e = DoTError::no_available_servers();
            return Err(e);
        }
    };
    let conn_string = format!("{}:{}", server.ip_address, server.port);

    // Create a new TLS connector and TCP stream and glue them together
    let connector = TlsConnector::new()?;

    let stream = TcpStream::connect(conn_string)?;

    let mut tls = connector.connect(server.hostname.as_str(), stream)?;

    // Write the serialized DNS request into the stream
    tls.write_all(msg)?;

    // Read back the 2 byte length field for the answer and deserialize it to
    // a number we can use
    let mut size_buff = vec![0; 2];
    tls.read_exact(&mut size_buff)?;
    let mut size_curs = Cursor::new(size_buff);
    let size = size_curs.read_u16::<NetworkEndian>()? as usize;

    // Make sure the answer isn't too large
    if size > 8192 {
        let e = DoTError::message_too_large();
        return Err(e);
    }

    // Create a buffer for the full answer and fill it
    let mut response = vec![0; size];
    tls.read_exact(&mut response)?;

    // Return the response buffer to the caller
    Ok(response)
}
