#![allow(dead_code)]

use goldfish::card::{Card, CardInstance, UNKNOWN_COST};
use goldfish::game::Game;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::thread;
use std::time;

const FILENAME: &str = r"C:\Program Files (x86)\Hearthstone\Logs\Power.log";

struct LogData {
    num_lines: usize,
    hand: Vec<CardInstance>,
    last_option_line: usize,
    mana: i32,
    opponent_damage: i32,
    opponent_armor: i32,
    last_create_game_line: usize,
}

fn read_log(last_create_game_line: usize) -> Result<LogData, std::io::Error> {
    let file_data = fs::read_to_string(FILENAME)?;
    let lines: Vec<_> = file_data.lines().collect();
    let mut log_data: LogData = LogData {
        num_lines: lines.len(),
        hand: Vec::new(),
        last_option_line: 0,
        mana: 0,
        opponent_damage: 0,
        opponent_armor: 0,
        last_create_game_line,
    };

    // Populate the id -> card_id map
    // This type of line happens whenever we draw a card, and perhaps at other times
    let mut card_id_map: BTreeMap<i32, String> = BTreeMap::new();

    // Map id -> changed cost.
    // This happens when we shadowstep, and perhaps other times.
    let mut cost_map: BTreeMap<i32, i32> = BTreeMap::new();

    // Figure out which player id we are. -1 means unknown.
    // We can also use this regex to find out where the game starts.
    let mut our_player_id = -1;
    let player_id_re = Regex::new(r"^.*PlayerID=(\d+), PlayerName=lacker#1660.*$").unwrap();

    let card_id_re = Regex::new(r"^.*Updating Entity.* id=(\d+) .* CardID=(\w+).*$").unwrap();
    let damage_re =
        Regex::new(r"^.*cardId=HERO_.*player=(\d+).*tag=DAMAGE value=(\d+).*$").unwrap();
    let armor_re = Regex::new(r"^.*cardId=HERO_.*player=(\d+).*tag=ARMOR value=(\d+).*$").unwrap();
    let cost_re =
        Regex::new(r"^.*TAG_CHANGE.*id=(\d+).*player=(\d+).*tag=COST value=(\d+).*$").unwrap();

    let skip_n = log_data.last_create_game_line;
    let enum_lines = || lines.iter().enumerate().skip(skip_n);

    for (i, line) in enum_lines() {
        if let Some(captures) = player_id_re.captures(line) {
            log_data.opponent_damage = 0;
            log_data.opponent_armor = 0;
            card_id_map.clear();
            cost_map.clear();
            log_data.last_create_game_line = i;
            our_player_id = captures[1].parse().unwrap();
        }
        if let Some(captures) = card_id_re.captures(line) {
            let id = captures[1].parse::<i32>().unwrap();
            let card_id = &captures[2];
            card_id_map.insert(id, card_id.to_string());
        }
        if let Some(captures) = damage_re.captures(line) {
            // println!("damage line: {}", line);
            let player_id = captures[1].parse::<i32>().unwrap();
            if our_player_id < 0 {
                println!("warning: unknown player id but saw damage line: {}", line);
            } else if player_id != our_player_id {
                log_data.opponent_damage = captures[2].parse::<i32>().unwrap();
            }
        }
        if let Some(captures) = armor_re.captures(line) {
            let player_id = captures[1].parse::<i32>().unwrap();
            if our_player_id < 0 {
                println!("warning: unknown player id but saw armor line");
            } else if player_id != our_player_id {
                log_data.opponent_armor = captures[2].parse::<i32>().unwrap();
            }
            log_data.opponent_armor = captures[1].parse::<i32>().unwrap();
        }
        if let Some(captures) = cost_re.captures(line) {
            let player_id = captures[1].parse::<i32>().unwrap();
            if our_player_id < 0 {
                println!("warning: unknown player id but saw cost line");
            } else if player_id != our_player_id {
                continue;
            }
            let id = captures[1].parse::<i32>().unwrap();
            let cost = captures[3].parse::<i32>().unwrap();
            cost_map.insert(id, cost);
        }
    }

    // Find the last option block
    let mut option_block: Vec<String> = Vec::new();
    let option_re = Regex::new(r"^.*GameState.DebugPrintOptions.*$").unwrap();
    for (i, line) in enum_lines().rev() {
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
    let unknown_card_re = Regex::new(r"^.*option.*type=POWER.* id=(\d+).*player=(\d)+.*$").unwrap();

    let mut handle_card = |id: i32, card: Card| {
        if seen_ids.contains(&id) {
            return;
        }
        seen_ids.insert(id);
        let mut ci = CardInstance::new(&card);
        match cost_map.get(&id) {
            Some(cost) => {
                if card.cost() != UNKNOWN_COST {
                    ci.cost_reduction = card.cost() - cost;
                }
            }
            None => (),
        }
        log_data.hand.push(ci);
    };

    for line in option_block.iter() {
        if known_card_re.is_match(line) {
            let caps = known_card_re.captures(line).unwrap();
            let name = &caps[1];
            let id = caps[2].parse::<i32>().unwrap();
            let c = Card::from_name(name);
            handle_card(id, c);
        } else if unknown_card_re.is_match(line) {
            let caps = unknown_card_re.captures(line).unwrap();
            let id = caps[1].parse::<i32>().unwrap();
            let player_id = caps[2].parse::<i32>().unwrap();
            if player_id != our_player_id {
                continue;
            }
            if let Some(card_id) = card_id_map.get(&id) {
                let c = Card::from_card_id(card_id);
                if c != Card::Unknown {
                    handle_card(id, c);
                }
            }
        }
    }

    // Find the mana
    let mana_re =
        Regex::new(r"^.*DebugPrintPower.*TAG_CHANGE Entity=lacker.*tag=RESOURCES value=(\d+).*$")
            .unwrap();
    for (_, line) in enum_lines() {
        if mana_re.is_match(line) {
            let caps = mana_re.captures(line).unwrap();
            log_data.mana = caps[1].parse().unwrap();
        }
    }

    Ok(log_data)
}

fn current_game(log_data: &LogData) -> Game {
    let mut game = Game::new();
    game.add_card_instances_to_hand(log_data.hand.clone().into_iter());
    game.mana = log_data.mana;
    game.life = 30 - log_data.opponent_damage + log_data.opponent_armor;
    game
}

fn main() {
    println!("watching");
    let mut previous_last_option_line = 0;
    let mut previous_last_create_game_line = 0;
    let mut last_mana = 0;
    loop {
        if let Ok(log_data) = read_log(previous_last_create_game_line) {
            if log_data.last_option_line > previous_last_option_line {
                let game = current_game(&log_data);
                if game.mana != last_mana {
                    println!("\nhand: {}", game.hand_string());
                    println!("mana: {}", log_data.mana);
                    println!("opponent life: {}", game.life);
                    game.print_plan();
                }
                last_mana = game.mana;
            }
            previous_last_option_line = log_data.last_option_line;
            previous_last_create_game_line = log_data.last_create_game_line;
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}
