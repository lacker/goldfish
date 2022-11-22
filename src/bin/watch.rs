#![allow(dead_code)]

use regex::Regex;
use std::fs;

const DIR: &str = r"C:\Program Files (x86)\Hearthstone\Logs";

fn scan() {
    let log_regex = Regex::new(r"^hearthstone_.*log$").unwrap();

    for entry in fs::read_dir(DIR).unwrap() {
        let entry = entry.unwrap();
        if !log_regex.is_match(entry.file_name().to_str().unwrap()) {
            continue;
        }
        println!("dir: {:?}", entry);
    }
}

fn main() {
    scan();
}
