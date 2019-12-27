use std::io::Cursor;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

use crate::error::TlsMessageError;

type Result<T> = std::result::Result<T, TlsMessageError>;


pub fn serialize(to_send: &[u8]) -> Result<Vec<u8>> {
    if to_send.len() == 0 {
        let e = TlsMessageError::bad_input_data();
        return Err(e);
    }

    let buff_size = to_send.len() + 2;
    let mut buffer = Vec::with_capacity(buff_size);

    // Write in the length of the buffer as two bytes
    buffer.write_u16::<NetworkEndian>(to_send.len() as u16)?;

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
        let e = TlsMessageError::bad_input_data();
        return Err(e);
    }
    let mut buffer = Vec::new();

    let mut rdr = Cursor::new(to_unwrap);
    let read_size = rdr.read_u16::<NetworkEndian>()?;

    // Copy the payload in
    buffer.extend_from_slice(to_unwrap);
    buffer.drain(0..2); // Remove the leading size bytes

    if read_size as usize != buffer.len() {
	let e = TlsMessageError::protocol_size_mismatch();
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
