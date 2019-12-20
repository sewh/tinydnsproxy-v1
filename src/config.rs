use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use toml;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BindDetails {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DnsServer {
    pub ip_address: String,
    pub port: u16,
    pub hostname: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub bind: BindDetails,
    pub dns_server: Vec<DnsServer>,
}

impl Config {
    pub fn from_toml(path: String) -> io::Result<Config> {
        let toml_contents = fs::read_to_string(path)?;
        let config = toml::from_str(toml_contents.as_str())?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::Write;
    use tempfile::NamedTempFile;

    extern crate tempfile;

    #[test]
    fn valid_file_works() {
        let f = r#"
[bind]
host = "0.0.0.0"
port = 53

[[dns_server]]
ip_address = "1.1.1.1"
port = 853
hostname = "cloudflare-dns.com"

[[dns_server]]
ip_address = "8.8.8.8"
port = 853
hostname = "dns.google"

"#;
        let servers = vec![
            DnsServer {
                ip_address: "1.1.1.1".to_string(),
                port: 853,
                hostname: "cloudflare-dns.com".to_string(),
            },
            DnsServer {
                ip_address: "8.8.8.8".to_string(),
                port: 853,
                hostname: "dns.google".to_string(),
            },
        ];

        let mut t = NamedTempFile::new().unwrap();
        t.write_all(f.as_bytes()).unwrap();

        let path = t.path().to_str().unwrap().to_string();
        let c = Config::from_toml(path).unwrap();

        assert_eq!(c.bind.host, "0.0.0.0");
        assert_eq!(c.bind.port, 53);

        let from_config: Vec<String> = c.dns_server.into_iter().map(|x| x.ip_address).collect();
        let already_done: Vec<String> = servers.into_iter().map(|x| x.ip_address).collect();

        assert_eq!(from_config, already_done);
    }
}
