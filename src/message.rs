use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use std::error;
use std::fmt;
use std::io::Cursor;

type Result<T> = std::result::Result<T, MessageError>;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum MessageErrorKind {
    NoData,
    TooSmall,
    ByteError,
    SizeMismatch,
}

#[derive(Debug, Clone)]
pub struct MessageError {
    why: MessageErrorKind,
}

impl MessageError {
    pub fn new(why: MessageErrorKind) -> MessageError {
        MessageError { why }
    }
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match &self.why {
            MessageErrorKind::NoData => "Buffer provided is empty",
            MessageErrorKind::TooSmall => {
                "Buffer provided is too small to be a DNS over TLS message"
            }
            MessageErrorKind::ByteError => "Could not coerce a number to or from a byte array",
            MessageErrorKind::SizeMismatch => {
                "Size in message is different to the actual size of the given buffer"
            }
        };
        write!(f, "{}", msg)
    }
}

impl error::Error for MessageError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub fn serialize(to_send: &[u8]) -> Result<Vec<u8>> {
    if to_send.len() == 0 {
        let e = MessageError::new(MessageErrorKind::NoData);
        return Err(e);
    }

    let buff_size = to_send.len() + 2;
    let mut buffer = Vec::with_capacity(buff_size);

    // Write in the length of the buffer as two bytes
    match buffer.write_u16::<NetworkEndian>(to_send.len() as u16) {
        Err(_) => {
            let e = MessageError::new(MessageErrorKind::ByteError);
            return Err(e);
        }
        _ => (),
    };

    // Append the payload to the buffer
    buffer.extend_from_slice(to_send);
    Ok(buffer)
}

#[allow(dead_code)]
pub fn deserialize(to_unwrap: &[u8]) -> Result<Vec<u8>> {
    // DoT messages are a two-byte length field followed by the payload.
    // If we don't have at least a two bytes of length and a single byte
    // as a payload, then we error here
    if to_unwrap.len() < 3 {
        let e = MessageError::new(MessageErrorKind::TooSmall);
        return Err(e);
    }
    let mut buffer = Vec::new();

    let mut rdr = Cursor::new(to_unwrap);
    let read_size = match rdr.read_u16::<NetworkEndian>() {
        Ok(s) => s,
        Err(_) => {
            let e = MessageError::new(MessageErrorKind::ByteError);
            return Err(e);
        }
    };

    // Copy the payload in
    buffer.extend_from_slice(to_unwrap);
    buffer.drain(0..2); // Remove the leading size bytes

    if read_size as usize != buffer.len() {
        let e = MessageError::new(MessageErrorKind::SizeMismatch);
        return Err(e);
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{NetworkEndian, ReadBytesExt};
    use std::io::Cursor;

    #[test]
    fn serialize_works() {
        let buff = b"Hello world";
        let orig_size = buff.len();

        let mut serialized = serialize(buff).unwrap();

        let mut rdr = Cursor::new(&serialized);
        let read_size = rdr.read_u16::<NetworkEndian>().unwrap();
        assert_eq!(orig_size, read_size as usize);

        serialized.drain(0..2);
        assert_eq!(buff.to_vec(), serialized);
    }

    #[test]
    fn serialize_deserialize_works() {
        let buff = b"Hello world".to_vec();
        let serialized = serialize(&buff).unwrap();
        let deserialized = deserialize(&serialized).unwrap();

        assert_eq!(buff, deserialized);
    }
}
