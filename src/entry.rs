use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use regex::Regex;

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.*?\D)(\d+)?(\..+?)?$").unwrap());
const SEPS: [char; 3] = ['_', '-', ' '];

pub struct Entry {
    p: PathBuf,
    b: String,
    a: Option<String>,
    n: Option<usize>,
}

impl From<PathBuf> for Entry {
    fn from(p: PathBuf) -> Self {
        let (b, a, n) = Self::parse_file_name(
            &p.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "".to_string()),
        );
        Self { p, b, a, n }
    }
}

impl Entry {
    fn parse_file_name(p: &str) -> (String, Option<String>, Option<usize>) {
        let c = RE.captures(&p);
        if let (Some(a), b, c) = (
            c.as_ref().and_then(|c| c.get(1)),
            c.as_ref().and_then(|c| c.get(2)),
            c.as_ref().and_then(|c| c.get(3)),
        ) {
            (
                a.as_str().to_string(),
                c.map(|m| m.as_str().to_string()),
                b.and_then(|b| b.as_str().parse().ok()),
            )
        } else {
            (p.to_string(), None, None)
        }
    }

    pub fn num(&self) -> Option<usize> {
        self.n
    }

    pub fn old_path(&self) -> &Path {
        &self.p
    }

    #[allow(dead_code)]
    pub fn file_name(&self) -> String {
        self.p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "".to_string())
    }

    pub fn file_name_before_sep(&self) -> String {
        self.b
            .strip_suffix(|_| self.file_name_sep().is_some())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.b.to_string())
    }

    pub fn file_name_sep(&self) -> Option<String> {
        self.b
            .chars()
            .last()
            .filter(|s| SEPS.contains(s))
            .map(|s| s.to_string())
    }

    pub fn new_path(&self, shift: bool, sep: Option<&str>, pad: usize) -> Option<PathBuf> {
        if let Some(n) = self.n {
            let mut p = self.p.to_path_buf();
            p.set_file_name(format!(
                "{}{:0w$}{}",
                self.b,
                if shift { n + 1 } else { n },
                self.a.as_ref().map(Cow::from).unwrap_or(Cow::from("")),
                w = pad
            ));
            Some(p)
        } else if let Some(sep) = sep {
            if shift {
                let mut p = self.p.to_path_buf();
                p.set_file_name(format!(
                    "{}{}{:0w$}{}",
                    self.b,
                    sep,
                    1,
                    self.a.as_ref().map(Cow::from).unwrap_or(Cow::from("")),
                    w = pad
                ));
                Some(p)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[test]
fn test_entry_parse_file_name() {
    assert_eq!(
        Entry::parse_file_name("aaaa.jpg"),
        ("aaaa".to_string(), Some(".jpg".to_string()), None)
    );
    assert_eq!(
        Entry::parse_file_name("aaaa_01"),
        ("aaaa_".to_string(), None, Some(1))
    );
    assert_eq!(
        Entry::parse_file_name("aaaa_1.jpg"),
        ("aaaa_".to_string(), Some(".jpg".to_string()), Some(1))
    );
    assert_eq!(
        Entry::parse_file_name("_001"),
        ("_".to_string(), None, Some(1))
    );
    assert_eq!(
        Entry::parse_file_name("001"),
        ("001".to_string(), None, None)
    );
}

#[test]
fn test_entry_file_name_before_sep() {
    assert_eq!(
        Entry {
            p: PathBuf::new(),
            b: "aaa_".to_string(),
            a: None,
            n: None,
        }
        .file_name_before_sep(),
        "aaa".to_string(),
    );
    assert_eq!(
        Entry {
            p: PathBuf::new(),
            b: "aaa".to_string(),
            a: None,
            n: None,
        }
        .file_name_before_sep(),
        "aaa".to_string(),
    );
}

#[test]
fn test_entry_file_name_sep() {
    assert_eq!(
        Entry {
            p: PathBuf::new(),
            b: "aaa_".to_string(),
            a: None,
            n: None,
        }
        .file_name_sep(),
        Some("_".to_string()),
    );
    assert_eq!(
        Entry {
            p: PathBuf::new(),
            b: "aaa".to_string(),
            a: None,
            n: None,
        }
        .file_name_sep(),
        None,
    );
}

#[test]
fn test_entry_new_path() {
    assert_eq!(
        Entry {
            p: PathBuf::new(),
            b: "aaa_".to_string(),
            a: Some(".txt".to_string()),
            n: Some(1),
        }
        .new_path(true, None, 1),
        Some(PathBuf::from("aaa_2.txt")),
    );
    assert_eq!(
        Entry {
            p: PathBuf::from("parent/aaa"),
            b: "aaa".to_string(),
            a: None,
            n: Some(3),
        }
        .new_path(true, None, 3),
        Some(PathBuf::from("parent/aaa004")),
    );
    assert_eq!(
        Entry {
            p: PathBuf::from("parent/aaa"),
            b: "aaa".to_string(),
            a: Some(".jpg".to_string()),
            n: None,
        }
        .new_path(true, Some("_"), 3),
        Some(PathBuf::from("parent/aaa_001.jpg")),
    );
}
