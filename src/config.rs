use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use toml;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockLists {
    pub refresh_after: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockList {
    pub list_type: String,
    pub format: String,
    pub path: Option<String>,
    pub url: Option<String>,
}

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
    pub block_lists: Option<BlockLists>,
    pub block_list: Vec<BlockList>,
    pub dns_server: Vec<DnsServer>,
}

impl Config {
    pub fn from_toml(path: String) -> io::Result<Config> {
        let toml_contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(toml_contents.as_str())?;

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

[block_lists]
refresh_after = 30

[[block_list]]
list_type = "file"
format = "hosts"
path = "/tmp/block.list"

[[block_list]]
list_type = "file"
format = "one-per-line"
path = "/tmp/block.2.list"

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

        let lists = vec![
            BlockList {
                list_type: "file".to_string(),
                format: "hosts".to_string(),
                path: Some("/tmp/block.list".to_string()),
                url: None,
            },
            BlockList {
                list_type: "file".to_string(),
                format: "one-per-line".to_string(),
                path: Some("/tmp/block.2.list".to_string()),
                url: None,
            },
        ];

        let mut t = NamedTempFile::new().unwrap();
        t.write_all(f.as_bytes()).unwrap();

        let path = t.path().to_str().unwrap().to_string();
        let c = Config::from_toml(path).unwrap();

        assert_eq!(c.bind.host, "0.0.0.0");
        assert_eq!(c.bind.port, 53);

        let servers_from_config: Vec<String> =
            c.dns_server.into_iter().map(|x| x.ip_address).collect();
        let servers_already_done: Vec<String> = servers.into_iter().map(|x| x.ip_address).collect();

        assert_eq!(servers_from_config, servers_already_done);

        for i in 0..lists.len() {
            let list_already_done = &lists[i];
            let list_from_config = &c.block_list[i];

            assert_eq!(list_already_done.list_type, list_from_config.list_type);
            assert_eq!(list_already_done.format, list_from_config.format);
            assert_eq!(list_already_done.path, list_from_config.path);
        }

        let block_lists = c.block_lists.unwrap();
        let refresh_after = block_lists.refresh_after.unwrap();
        assert_eq!(refresh_after, 30);
    }
}
