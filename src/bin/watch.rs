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
    opponent_damage: i32,
}

fn read_log() -> Result<LogData, std::io::Error> {
    let file_data = fs::read_to_string(FILENAME)?;
    let lines: Vec<_> = file_data.lines().collect();
    let mut log_data: LogData = LogData {
        num_lines: lines.len(),
        hand: Vec::new(),
        last_option_line: 0,
        mana: 0,
        opponent_damage: 0,
    };

    // Populate the id -> card_id map
    // This type of line happens whenever we draw a card, and perhaps at other times
    let mut card_id_map: BTreeMap<i32, String> = BTreeMap::new();
    let card_id_re = Regex::new(r"^.*Updating Entity.* id=(\d+) .* CardID=(\w+).*$").unwrap();
    let damage_re = Regex::new(r"^.*cardId=HERO_.*player=2.*tag=DAMAGE value=(\d+).*$").unwrap();
    for line in lines.iter() {
        if let Some(captures) = card_id_re.captures(line) {
            let id = captures[1].parse::<i32>().unwrap();
            let card_id = &captures[2];
            card_id_map.insert(id, card_id.to_string());
        }
        if let Some(captures) = damage_re.captures(line) {
            log_data.opponent_damage = captures[1].parse::<i32>().unwrap();
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
        Regex::new(r"^.*option.*type=POWER.*entityName=UNKNOWN ENTITY.* id=(\d+).*player=1.*$")
            .unwrap();
    for line in option_block.iter() {
        if known_card_re.is_match(line) {
            // println!("{}", line);
            let caps = known_card_re.captures(line).unwrap();
            let id = caps[2].parse::<i32>().unwrap();
            if seen_ids.contains(&id) {
                continue;
            }
            seen_ids.insert(id);
            let name = &caps[1];
            let c = Card::from_name(name);
            if c == Card::Unknown {
                println!("unknown card name: {}", name);
            }
            log_data.hand.push(c);
        } else if unknown_card_re.is_match(line) {
            // println!("{}", line);
            let caps = unknown_card_re.captures(line).unwrap();
            let id = caps[1].parse::<i32>().unwrap();
            if seen_ids.contains(&id) {
                continue;
            }
            seen_ids.insert(id);
            if let Some(card_id) = card_id_map.get(&id) {
                let c = Card::from_card_id(card_id);
                if c == Card::Unknown {
                    println!("unknown id : {} card id: {}", id, card_id);
                }
                log_data.hand.push(c);
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

fn current_game(log_data: &LogData) -> Game {
    let mut game = Game::new();
    game.add_cards_to_hand(log_data.hand.clone().into_iter());
    game.mana = log_data.mana;
    game.life = 30 - log_data.opponent_damage;
    game
}

fn main() {
    println!("watching");
    let mut previous_last_option_line = 0;
    let mut last_mana = 0;
    loop {
        if let Ok(log_data) = read_log() {
            if log_data.last_option_line > previous_last_option_line {
                let game = current_game(&log_data);
                if game.mana != last_mana {
                    println!("\nhand: {:?}", log_data.hand);
                    println!("mana: {}", log_data.mana);
                    println!("opponent life: {}", game.life);
                    game.print_plan();
                }
                last_mana = game.mana;
            }
            previous_last_option_line = log_data.last_option_line;
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}
