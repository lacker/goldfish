#![allow(dead_code)]

use regex::Regex;
use std::fs;
use std::thread;
use std::time;

const FILENAME: &str = r"C:\Program Files (x86)\Hearthstone\Logs\Power.log";

struct LogData {
    num_lines: usize,
    option_block: Vec<String>,
    last_option_line: usize,
    mana: i32,
}

// Prints out any lines past the previous number of lines
fn read_log(previous_num_lines: usize) -> Result<LogData, std::io::Error> {
    let file_data = fs::read_to_string(FILENAME)?;
    let lines: Vec<_> = file_data.lines().collect();
    let mut log_data: LogData = LogData {
        num_lines: lines.len(),
        option_block: Vec::new(),
        last_option_line: 0,
        mana: 0,
    };
    for line in &lines[previous_num_lines..] {
        println!("{}", line);
    }

    // Find the last option block
    let option_re = Regex::new(r"^.*GameState.DebugPrintOptions.*$").unwrap();
    for (i, line) in lines.iter().enumerate().rev() {
        if option_re.is_match(line) {
            if log_data.option_block.is_empty() {
                log_data.last_option_line = i;
            }
            log_data.option_block.push(line.to_string());
        } else if !log_data.option_block.is_empty() {
            // We've reached the end of the option block
            break;
        }
    }
    log_data.option_block.reverse();

    // Find the mana
    let mana_re =
        Regex::new(r"^.*DebugPrintPower.*TAG_CHANGE Entity=lacker.*tag=RESOURCES value=(\d+).*$")
            .unwrap();
    for line in lines.iter() {
        if mana_re.is_match(line) {
            let caps = mana_re.captures(line).unwrap();
            log_data.mana = caps[1].parse().unwrap();
        }
    }

    Ok(log_data)
}

fn main() {
    println!("watching");
    let mut previous_num_lines = 0;
    let mut previous_last_option_line = 0;
    loop {
        if let Ok(log_data) = read_log(previous_num_lines) {
            if log_data.last_option_line > previous_last_option_line {
                println!("\nnew option block:");
                for line in &log_data.option_block {
                    println!("{}", line);
                }
                println!("mana: {}\n", log_data.mana);
            }
            previous_num_lines = log_data.num_lines;
            previous_last_option_line = log_data.last_option_line;
        }
        thread::sleep(time::Duration::from_secs(5));
    }
}
