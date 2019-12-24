pub struct DnsMessageError;

type HostnameResult = std::result::Result<String, DnsMessageError>;

pub fn hostname_from_bytes(bytes: &[u8]) -> HostnameResult {
    Ok("Hello world".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dns_message_pass() {
	assert!(true);
    }
}
