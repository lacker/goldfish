#![allow(dead_code)]

use std::fmt;
use std::iter;
use std::time::Instant;

mod card;
use card::Card;

// Properties that apply to only the specific version of this card, in our hand.
// This could extend to on-board properties later.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct CardInstance {
    card: Card,
    potion: bool,
    tenwu: bool,
}

impl fmt::Display for CardInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.card.to_string()).unwrap();
        if self.potion {
            f.write_str(" (potion)").unwrap();
        }
        if self.tenwu {
            f.write_str(" (tenwu)").unwrap();
        }
        Ok(())
    }
}

impl CardInstance {
    fn new(card: &Card) -> Self {
        Self {
            card: *card,
            potion: false,
            tenwu: false,
        }
    }

    fn cost(&self) -> i32 {
        if self.potion || self.tenwu {
            1
        } else {
            self.card.cost()
        }
    }
}

#[derive(Clone)]
struct Game {
    board: Vec<Card>,        // our side of the board
    hand: Vec<CardInstance>, // our hand
    life: i32,               // the opponent's life
    mana: i32,               // our mana
    storm: i32,              // number of things played this turn
    foxy: i32,               // number of stacks of the foxy effect
    scabbs: i32,             // number of stacks of the scabbs effect
    next_scabbs: i32,        // number of stacks of the scabbs effect after this one
}

type Move = usize; // which card in hand to play

enum Plan {
    Win(Vec<Move>),
    Lose,
    Timeout,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "board: {}",
            self.board
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
        .unwrap();
        writeln!(
            f,
            "hand: {}",
            self.hand
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
        .unwrap();
        writeln!(f, "life: {}", self.life).unwrap();
        writeln!(f, "mana: {}", self.mana).unwrap();
        if self.storm > 0 {
            writeln!(f, "storm: {}", self.storm).unwrap();
        }
        if self.foxy > 0 {
            writeln!(f, "foxy: {}", self.foxy).unwrap();
        }
        if self.scabbs > 0 {
            writeln!(f, "scabbs: {}", self.scabbs).unwrap();
        }
        if self.next_scabbs > 0 {
            writeln!(f, "next_scabbs: {}", self.next_scabbs).unwrap();
        }
        Ok(())
    }
}

impl Game {
    fn new() -> Self {
        Self {
            board: Vec::new(),
            hand: Vec::new(),
            life: 30,
            mana: 0,
            storm: 0,
            foxy: 0,
            scabbs: 0,
            next_scabbs: 0,
        }
    }

    // Mana cost of the card at the given index in hand
    // Handles discounts
    fn cost(&self, index: usize) -> i32 {
        let card = self.hand[index];
        let mut cost = card.cost() - self.scabbs * 3;
        if card.card.combo() {
            cost -= self.foxy * 2;
        }
        std::cmp::max(cost, 0)
    }

    // Whether we can play the card at the given index in hand
    fn can_play(&self, index: usize) -> bool {
        let card = self.hand[index];
        if self.board.len() >= 7 && card.card.minion() {
            // The board is full
            return false;
        }
        self.mana >= self.cost(index)
    }

    fn add_card_instances_to_hand(&mut self, iter: impl Iterator<Item = CardInstance>) {
        for ci in iter {
            if self.hand.len() >= 10 {
                break;
            }
            self.hand.push(ci);
        }
        self.hand.sort();
    }

    fn add_card_instance_to_hand(&mut self, ci: CardInstance) {
        self.add_card_instances_to_hand(iter::once(ci))
    }

    fn add_cards_to_hand(&mut self, iter: impl Iterator<Item = Card>) {
        self.add_card_instances_to_hand(iter.map(|c| CardInstance::new(&c)))
    }

    fn add_card_to_hand(&mut self, card: &Card) {
        self.add_card_instance_to_hand(CardInstance::new(card))
    }

    // Handles battlecries and combos
    fn come_into_play(&mut self, card: &Card) {
        match card {
            Card::Dancer => self.add_card_to_hand(&Card::Coin),
            Card::Foxy => self.foxy += 1,
            Card::Pillager => self.life -= self.storm,
            Card::Scabbs => {
                if self.storm > 0 {
                    self.scabbs += 1;
                    self.next_scabbs += 1;
                }
            }
            _ => (),
        }
    }

    // Play the card at the given index in hand
    fn play(&mut self, index: usize) {
        let card = self.hand[index];
        self.mana -= self.cost(index);
        self.hand.remove(index);
        self.foxy = 0;
        self.scabbs = self.next_scabbs;
        self.next_scabbs = 0;

        if card.card.minion() {
            self.board.push(card.card);
        }

        self.come_into_play(&card.card);
        if self.board.contains(&Card::Shark) {
            self.come_into_play(&card.card);
        }

        match card.card {
            Card::Coin => self.mana += 1,
            Card::Potion => {
                let cis: Vec<CardInstance> = self
                    .board
                    .iter()
                    .map(|c| {
                        let mut ci = CardInstance::new(c);
                        ci.potion = true;
                        ci
                    })
                    .collect();
                self.add_card_instances_to_hand(cis.into_iter());
            }
            _ => (),
        }

        self.storm += 1
    }

    fn possible_moves(&self) -> Vec<usize> {
        (0..self.hand.len()).filter(|i| self.can_play(*i)).collect()
    }

    fn is_win(&self) -> bool {
        self.life <= 0
    }

    // Returns a plan with reversed moves.
    fn find_win_helper(&self, start: Instant) -> Plan {
        if start.elapsed().as_secs() > 5 {
            return Plan::Timeout;
        }
        if self.is_win() {
            return Plan::Win(Vec::new());
        }
        let possible = self.possible_moves();
        for m in possible {
            let mut clone = self.clone();
            clone.play(m);
            match clone.find_win_helper(start) {
                Plan::Win(mut moves) => {
                    moves.push(m);
                    return Plan::Win(moves);
                }
                Plan::Lose => (),
                Plan::Timeout => return Plan::Timeout,
            }
        }

        // Our search is exhausted
        Plan::Lose
    }

    // Returns a plan with list of moves to win.
    fn find_win(&self) -> Plan {
        let start = Instant::now();
        match self.find_win_helper(start) {
            Plan::Win(mut moves) => {
                moves.reverse();
                Plan::Win(moves)
            }
            x => x,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game() {
        let game = Game::new();
        assert!(game.hand.len() == 0)
    }

    #[test]
    fn making_a_dancer() {
        let c: Card = Card::Dancer;
        assert!(c.cost() == 2);
        assert!(c.minion() == true);
        assert!(c.combo() == false);
    }
}

fn main() {
    println!("TODO: something");
}
