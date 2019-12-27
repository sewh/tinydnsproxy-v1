use std::error;
use std::fmt;

#[derive(Debug)]
pub enum BlockListErrorKind {
    Io(std::io::Error),
    NoEntries,
}

#[derive(Debug)]
pub struct BlockListError {
    kind: BlockListErrorKind
}

impl BlockListError {
    pub fn new(kind: BlockListErrorKind) -> Self {
	BlockListError {kind}
    }

    pub fn no_entries() -> Self {
	BlockListError::new(BlockListErrorKind::NoEntries)
    }
}

impl fmt::Display for BlockListError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	use BlockListErrorKind::*;

	let suffix = match &self.kind {
	    Io(e) => format!("{}", e),
	    NoEntries => "No block list entries".to_string(),
	};
	write!(f, "Block list error: {}", suffix)
    } 
}

impl error::Error for BlockListError {}

impl From<std::io::Error> for BlockListError {
    fn from(e: std::io::Error) -> Self {
	BlockListError::new(BlockListErrorKind::Io(e))
    }
}

#[derive(Debug)]
pub enum DnsMessageErrorKind {
    Io(std::io::Error),
    StringEncoding(std::string::FromUtf8Error),
    TooManyQuestions,
    UnexpectedReadLength,
}

#[derive(Debug)]
pub struct DnsMessageError {
    kind: DnsMessageErrorKind
}

impl DnsMessageError {
    pub fn new(kind: DnsMessageErrorKind) -> Self {
	DnsMessageError {kind}
    }

    pub fn too_many_questions() -> Self {
	let k = DnsMessageErrorKind::TooManyQuestions;

	DnsMessageError::new(k)
    }

    pub fn unexpected_read_length() -> Self {
	let k = DnsMessageErrorKind::UnexpectedReadLength;

	DnsMessageError::new(k)
    }
}

impl fmt::Display for DnsMessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	use DnsMessageErrorKind::*;

	let suffix = match &self.kind {
	    Io(e) => format!("{}", e),
	    StringEncoding(e) => format!("{}", e),
	    TooManyQuestions => "Too many DNS questions in request".to_string(),
	    UnexpectedReadLength => "Read an unexpected amount of data".to_string()
	};
	write!(f, "DNS Message Parsing Error: {}", suffix)
    }
}

impl error::Error for DnsMessageError {}

impl From<std::io::Error> for DnsMessageError {
    fn from(e: std::io::Error) -> Self {
	let kind = DnsMessageErrorKind::Io(e);

	DnsMessageError::new(kind)
    }
}

impl From<std::string::FromUtf8Error> for DnsMessageError {
    fn from(e: std::string::FromUtf8Error) -> Self {
	let kind = DnsMessageErrorKind::StringEncoding(e);

	DnsMessageError::new(kind)
    }
}

#[derive(Debug)]
pub enum TlsMessageErrorKind {
    Io(std::io::Error),
    ProtocolSizeMismatch,
    BadInputData,
}

#[derive(Debug)]
pub struct TlsMessageError {
    kind: TlsMessageErrorKind
}

impl TlsMessageError {
    pub fn new(kind: TlsMessageErrorKind) -> Self {
	TlsMessageError {kind}
    }

    pub fn protocol_size_mismatch() -> Self {
	let k = TlsMessageErrorKind::ProtocolSizeMismatch;
	TlsMessageError::new(k)
    }

    pub fn bad_input_data() -> Self {
	let k = TlsMessageErrorKind::BadInputData;
	TlsMessageError::new(k)
    }
}

impl fmt::Display for TlsMessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	use TlsMessageErrorKind::*;

	let suffix = match &self.kind {
	    Io(e) => format!("{}", e),
	    ProtocolSizeMismatch => "Received a message that was a different size to what the protocol said it should be".to_string(),
	    BadInputData => "Input buffer is incorrect".to_string(),
	};
	write!(f, "DNS-over-TLS Message Error: {}", suffix)
    }
}

impl error::Error for TlsMessageError {}

impl From<std::io::Error> for TlsMessageError {
    fn from(e: std::io::Error) -> Self {
	let k = TlsMessageErrorKind::Io(e);
	TlsMessageError::new(k)
    }
}
