#![allow(dead_code)]

use regex::Regex;
use std::fs;

const DIR: &str = r"C:\Program Files (x86)\Hearthstone\Logs";

fn is_log_file(entry: &fs::DirEntry) -> bool {
    let re = Regex::new(r"^hearthstone_.*log$").unwrap();
    re.is_match(entry.file_name().to_str().unwrap())
}

fn last_log_file() -> Option<fs::DirEntry> {
    let mut entries: Vec<_> = fs::read_dir(DIR)
        .unwrap()
        .map(|e| e.unwrap())
        .filter(is_log_file)
        .collect();
    entries.sort_by_key(|e| e.path());
    entries.into_iter().last()
}

fn scan() {
    match last_log_file() {
        Some(entry) => {
            println!("log file: {}", entry.path().to_str().unwrap());
        }
        None => println!("No log files found"),
    }
}

fn main() {
    scan();
}
