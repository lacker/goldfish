#![allow(dead_code)]

use goldfish::card::Card;
use goldfish::game::Game;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::thread;
use std::time;

const FILENAME: &str = r"C:\Program Files (x86)\Hearthstone\Logs\Power.log";

struct LogData {
    num_lines: usize,
    hand: Vec<Card>,
    last_option_line: usize,
    mana: i32,
}

// Prints out any lines past the previous number of lines
fn read_log() -> Result<LogData, std::io::Error> {
    let file_data = fs::read_to_string(FILENAME)?;
    let lines: Vec<_> = file_data.lines().collect();
    let mut log_data: LogData = LogData {
        num_lines: lines.len(),
        hand: Vec::new(),
        last_option_line: 0,
        mana: 0,
    };

    // Populate the id -> card_id map
    // This sort of line happens
    let mut card_id_map: BTreeMap<i32, String> = BTreeMap::new();
    let card_id_re = Regex::new(r"^.*Updating Entity.* id=(\d+) .* CardID=(\w+).*$").unwrap();
    for line in lines.iter() {
        if let Some(captures) = card_id_re.captures(line) {
            let id = captures[1].parse::<i32>().unwrap();
            let card_id = &captures[2];
            card_id_map.insert(id, card_id.to_string());
        }
    }

    // Find the last option block
    let mut option_block: Vec<String> = Vec::new();
    let option_re = Regex::new(r"^.*GameState.DebugPrintOptions.*$").unwrap();
    for (i, line) in lines.iter().enumerate().rev() {
        if option_re.is_match(line) {
            if option_block.is_empty() {
                log_data.last_option_line = i;
            }
            option_block.push(line.to_string());
        } else if !option_block.is_empty() {
            // We've reached the end of the option block
            break;
        }
    }
    option_block.reverse();

    // Extract the hand
    let mut seen_ids: BTreeSet<i32> = BTreeSet::new();
    let known_card_re =
        Regex::new(r"^.*entityName=([^=]+) id=(\d+) zone=(?:HAND|SETASIDE).*$").unwrap();
    let unknown_card_re =
        Regex::new(r"^.*option.*type=POWER.*entityName=UNKNOWN ENTITY.* id=(\d+).*$").unwrap();
    for line in option_block.iter() {
        if known_card_re.is_match(line) {
            println!("{}", line);
            let caps = known_card_re.captures(line).unwrap();
            let id = caps[2].parse::<i32>().unwrap();
            if seen_ids.contains(&id) {
                continue;
            }
            seen_ids.insert(id);
            log_data.hand.push(Card::from_name(&caps[1]));
        } else if unknown_card_re.is_match(line) {
            let caps = unknown_card_re.captures(line).unwrap();
            let id = caps[1].parse::<i32>().unwrap();
            if let Some(card_id) = card_id_map.get(&id) {
                log_data.hand.push(Card::from_card_id(card_id));
            }
        }
    }

    // Find the mana
    let mana_re =
        Regex::new(r"^.*DebugPrintPower.*TAG_CHANGE Entity=lacker.*tag=RESOURCES value=(\d+).*$")
            .unwrap();
    for line in lines {
        if mana_re.is_match(line) {
            let caps = mana_re.captures(line).unwrap();
            log_data.mana = caps[1].parse().unwrap();
        }
    }

    Ok(log_data)
}

fn suggest(log_data: &LogData) {
    println!("\nhand: {:?}", log_data.hand);
    println!("mana: {}", log_data.mana);

    let mut game = Game::new();
    game.add_cards_to_hand(log_data.hand.clone().into_iter());
    game.mana = log_data.mana;
    game.print_plan();
}

fn main() {
    println!("watching");
    let mut previous_last_option_line = 0;
    loop {
        if let Ok(log_data) = read_log() {
            if log_data.last_option_line > previous_last_option_line {
                suggest(&log_data);
            }
            previous_last_option_line = log_data.last_option_line;
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}
