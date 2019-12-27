use std::io::Cursor;
use std::io::prelude::*;
use std::io::SeekFrom;
use byteorder::{NetworkEndian, ReadBytesExt};

pub struct DnsMessageError;

impl DnsMessageError {
    pub fn new() -> DnsMessageError {
	DnsMessageError {}
    }
}

impl std::fmt::Debug for DnsMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
	write!(f, "DNS Message Parsing Error")
    }
}

type HostnameResult = std::result::Result<String, DnsMessageError>;
type NxDomainResult = std::result::Result<Vec<u8>, DnsMessageError>;

pub fn hostname_from_bytes(bytes: &[u8]) -> HostnameResult {
    let mut cursor = Cursor::new(bytes);

    // Skip to where the questions are
    match cursor.seek(SeekFrom::Start(4)) {
	Err(_) => {
	    let e = DnsMessageError::new();
	    return Err(e);
	},
	_ => ()
    };

    let questions = match cursor.read_u16::<NetworkEndian>() {
	Ok(q) => q,
	Err(_) => {
	    let e = DnsMessageError::new();
	    return Err(e);	    
	}
    };

    if questions != 1 {
	// We don't want the hastle of rewriting DNS querys (yet) so
	// if we have more than one question, we'll just throw an error
	let e = DnsMessageError::new();
	return Err(e);
    }

    // Now skip to the first question
    match cursor.seek(SeekFrom::Start(12)) {
	Err(_) => {
	    let e = DnsMessageError::new();
	    return Err(e);
	},
	_ => ()
    };

    // Now read the string in DNS format
    let mut hostname_buff: Vec<u8> = Vec::new();
    loop {
	let mut size_buff = vec![0; 1];
	if let Ok(result) = cursor.read(&mut size_buff) {
	    if result != 1 {
		let e = DnsMessageError::new();
		return Err(e);
	    }
	}

	if size_buff[0] == 0x0 {
	    // Remove last dot and then bail
	    let new_size = hostname_buff.len() - 1;
	    hostname_buff.truncate(new_size);
	    break;
	}

	for _ in 0..size_buff[0] {
	    let mut byte_buff = vec![0; 1];
	    if let Ok(result) = cursor.read(&mut byte_buff) {
		if result != 1 {
		    let e = DnsMessageError::new();
		    return Err(e);
		}
	    }
	    hostname_buff.push(byte_buff[0]);
	}

	hostname_buff.push('.' as u8);

    }
    
    let as_string = match String::from_utf8(hostname_buff) {
	Ok(s) => s,
	Err(_) => {
	    let e = DnsMessageError::new();
	    return Err(e);	    
	}
    };

    Ok(as_string)
}

pub fn create_nxdomain(request: &[u8]) -> NxDomainResult {
    let mut output = Vec::from(request);
    let mut cursor = Cursor::new(&mut output);
    let response_bytes = vec![0x81, 0x83];

    // We replace the flags to make it look like NXDomain
    match cursor.seek(SeekFrom::Start(2)) {
	Err(_) => {
	    let e = DnsMessageError::new();
	    return Err(e);
	},
	_ => (),
    };

    match cursor.write(&response_bytes) {
	Err(_) => {
	    let e = DnsMessageError::new();
	    return Err(e);
	},
	_ => (),
    };

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostname_from_bytes_works() {
        let msg: Vec<u8> = vec![
            0xe4, 0x72, 0x01, 0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x04, 0x6d,
            0x61, 0x69, 0x6c, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d,
            0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x29, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
	let expected: String = "mail.google.com".to_string();

	let hostname_res = hostname_from_bytes(&msg);
	assert!(hostname_res.is_ok());

	let hostname = hostname_res.unwrap();
	assert_eq!(hostname, expected);
    }
}
