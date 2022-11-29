#![allow(dead_code)]

use rand;
use std::fmt;
use std::iter;
use std::time::Instant;

use crate::card::Card;
use crate::card::CardInstance;
use crate::card::STARTING_DECK;

#[derive(Clone)]
pub struct Game {
    pub board: Vec<Card>,        // our side of the board
    pub hand: Vec<CardInstance>, // our hand
    pub life: i32,               // the opponent's life
    pub mana: i32,               // our current mana
    storm: i32,                  // number of things played this turn
    foxy: i32,                   // number of stacks of the foxy effect
    scabbs: i32,                 // number of stacks of the scabbs effect
    next_scabbs: i32,            // number of stacks of the scabbs effect after this one
    pub deck: Vec<Card>,         // the cards left in the deck
    pub turn: i32,               // the current turn
    prep_pending: bool,          // whether we have a preparation effect pending
}

pub struct Move {
    index: usize,          // which card in hand to play
    target: Option<usize>, // which card on the board to target
}

pub enum Plan {
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
        )?;
        writeln!(f, "hand: {}", self.hand_string())?;
        writeln!(f, "life: {}", self.life)?;
        writeln!(f, "mana: {}", self.mana)?;
        if self.storm > 0 {
            writeln!(f, "storm: {}", self.storm)?;
        }
        if self.foxy > 0 {
            writeln!(f, "foxy: {}", self.foxy)?;
        }
        if self.scabbs > 0 {
            writeln!(f, "scabbs: {}", self.scabbs)?;
        }
        if self.next_scabbs > 0 {
            writeln!(f, "next_scabbs: {}", self.next_scabbs)?;
        }
        Ok(())
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Vec::new(),
            hand: Vec::new(),
            life: 30,
            mana: 0,
            storm: 0,
            foxy: 0,
            scabbs: 0,
            next_scabbs: 0,
            deck: STARTING_DECK.to_vec(),
            turn: 0,
            prep_pending: false,
        }
    }

    pub fn hand_string(&self) -> String {
        self.hand
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn move_string(&self, m: &Move) -> String {
        let mut s = self.hand[m.index].to_string();
        if let Some(t) = m.target {
            s.push_str(" -> ");
            s.push_str(&self.board[t].to_string());
        }
        s
    }

    // Mana cost of the card at the given index in hand
    // Handles discounts
    fn cost(&self, index: usize) -> i32 {
        let card = self.hand[index];
        let mut cost = card.cost() - self.scabbs * 3;
        if card.card.combo() {
            cost -= self.foxy * 2;
        }
        if card.card.spell() && self.prep_pending {
            cost -= 2;
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

    pub fn add_cards_to_hand(&mut self, iter: impl Iterator<Item = Card>) {
        self.add_card_instances_to_hand(iter.map(|c| CardInstance::new(&c)))
    }

    pub fn add_card_to_hand(&mut self, card: &Card) {
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

    // Draws one random card into our hand
    fn draw(&mut self) -> bool {
        if self.deck.is_empty() {
            return false;
        }
        let i = rand::random::<usize>() % self.deck.len();
        let card = self.deck.remove(i);
        self.add_card_to_hand(&card);
        true
    }

    pub fn new_going_first() -> Self {
        let mut game = Self::new();
        game.draw();
        game.draw();
        game.draw();
        game
    }

    pub fn new_going_second() -> Self {
        let mut game = Self::new();
        game.draw();
        game.draw();
        game.draw();
        game.draw();
        game.add_card_to_hand(&Card::Coin);
        game
    }

    // Play the card at the given index in hand
    pub fn make_move(&mut self, m: &Move) {
        let card = self.hand[m.index];
        self.mana -= self.cost(m.index);
        self.hand.remove(m.index);
        self.foxy = 0;
        self.scabbs = self.next_scabbs;
        self.next_scabbs = 0;

        if card.card == Card::Tenwu {
            let target_card = self.board[m.target.unwrap()];
            let mut ci = CardInstance::new(&target_card);
            ci.tenwu = true;
            self.add_card_instance_to_hand(ci);
        }

        if card.card.minion() {
            self.board.push(card.card);
        } else if card.card.spell() {
            self.prep_pending = false;
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
            Card::Preparation => self.prep_pending = true,
            Card::Shadowstep => {
                let target_card = self.board.remove(m.target.unwrap());
                let mut ci = CardInstance::new(&target_card);
                ci.cost_reduction = 2;
                self.add_card_instance_to_hand(ci);
            }
            _ => (),
        }

        self.storm += 1
    }

    fn possible_moves(&self) -> Vec<Move> {
        let mut answer = Vec::new();
        for (index, ci) in self.hand.iter().enumerate() {
            if !self.can_play(index) {
                continue;
            }
            if ci.card.must_target() {
                for target in 0..self.board.len() {
                    answer.push(Move {
                        index,
                        target: Some(target),
                    })
                }
            } else {
                answer.push(Move {
                    index,
                    target: None,
                });
            }
        }
        answer
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
            clone.make_move(&m);
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
    pub fn find_win(&self) -> Plan {
        let start = Instant::now();
        match self.find_win_helper(start) {
            Plan::Win(mut moves) => {
                moves.reverse();
                Plan::Win(moves)
            }
            x => x,
        }
    }

    pub fn next_turn(&mut self) {
        self.turn += 1;
        self.mana = std::cmp::min(self.turn, 10);
        self.storm = 0;
        for ci in self.hand.iter_mut() {
            ci.tenwu = false;
        }
        self.draw();
    }

    // Returns whether we won or not.
    pub fn print_plan(&self) -> bool {
        let plan = self.find_win();
        match plan {
            Plan::Win(moves) => {
                println!("win found:");
                let mut clone = self.clone();
                for m in moves {
                    println!("{}", clone.move_string(&m));
                    clone.make_move(&m);
                }
                true
            }
            Plan::Lose => {
                println!("cannot win");
                false
            }
            Plan::Timeout => {
                println!("timeout, no win found");
                false
            }
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
