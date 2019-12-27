use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::error::BlockListError;

pub type Result = std::result::Result<(), BlockListError>;

#[derive(Clone, Debug)]
pub enum BlockListKind {
    File,
}

#[derive(Clone, Debug)]
pub enum BlockListFormat {
    Hosts,
    OnePerLine,
}

#[derive(Clone, Debug)]
pub struct BlockList {
    pub kind: BlockListKind,
    pub format: BlockListFormat,
    pub path: Option<String>,
    pub url: Option<String>,
    pub entries: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct BlockLists {
    pub lists: Vec<BlockList>,
}

impl BlockLists {
    pub fn new() -> BlockLists {
        let lists = Vec::new();
        BlockLists { lists }
    }

    pub fn add_file(&mut self, path: &String, format: &BlockListFormat) -> Result {
        let mut entries: Vec<String> = Vec::new();

        let file = File::open(path)?;

        let reader = BufReader::new(file);

        for (_index, line_res) in reader.lines().enumerate() {
            if let Ok(line) = line_res {
                if let Some(processed_line) = process_line(&line, &format) {
                    entries.push(processed_line);
                }
            }
        }

        if entries.len() == 0 {
            return Err(BlockListError::no_entries());
        }

        let list = BlockList {
            kind: BlockListKind::File,
            format: format.clone(),
            path: Some(path.clone()),
            url: None,
            entries: entries,
        };

        self.lists.push(list);

        Ok(())
    }

    pub fn is_blocked(&self, hostname: &String) -> bool {
        for list in &self.lists {
            for entry in &list.entries {
                if entry == hostname {
                    return true;
                }
            }
        }

        false
    }
}

fn process_line(line: &String, format: &BlockListFormat) -> Option<String> {
    let no_comments = match strip_comments(line) {
        Some(s) => s,
        None => return None,
    };

    if let BlockListFormat::Hosts = format {
        if let Some(host) = extract_hostname(&no_comments) {
            return Some(host);
        } else {
            return None;
        }
    } else if let BlockListFormat::OnePerLine = format {
        return Some(no_comments);
    } else {
        return None;
    }
}

fn strip_comments(line: &String) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"((^|\s+)#.+)"#).unwrap();
    }
    let no_comments = RE.replace(line, "").to_mut().to_string();

    if no_comments == "" {
        return None;
    }

    Some(no_comments)
}

fn extract_hostname(line: &String) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^.+\s+"#).unwrap();
    }

    let hostname = RE.replace(line, "").to_mut().to_string();
    if hostname == "" {
        return None;
    }

    Some(hostname)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_comments_works() {
        let line1 = "# A comment here".to_string();
        let line2 = "Something here # but now a comment".to_string();
        let line2_correct = "Something here".to_string();

        let line1_res = strip_comments(&line1);
        assert!(line1_res.is_none());

        let line2_res = strip_comments(&line2).unwrap();
        assert_eq!(line2_correct, line2_res);
    }

    #[test]
    fn extract_hostname_works() {
        let line1 = "127.0.0.1 google.com".to_string();
        let line1_correct = "google.com".to_string();
        let line2 = "8.8.8.8 dns.google".to_string();
        let line2_correct = "dns.google".to_string();
        let line3 = "".to_string();

        let res1 = extract_hostname(&line1).unwrap();
        let res2 = extract_hostname(&line2).unwrap();
        let res3 = extract_hostname(&line3);

        assert_eq!(res1, line1_correct);
        assert_eq!(res2, line2_correct);
        assert!(res3.is_none());
    }
}
