use rand;
use rand::seq::{IteratorRandom, SliceRandom};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter;
use std::time::Instant;

use crate::card::Card;
use crate::card::CardInstance;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Game {
    pub board: Vec<Card>,           // our side of the board
    pub hand: Vec<CardInstance>,    // our hand
    pub passage: Vec<CardInstance>, // cards we've set aside with Secret Passage
    pub life: i32,                  // the opponent's life
    pub mana: i32,                  // our current mana
    storm: i32,                     // number of things played this turn
    foxy: i32,                      // number of stacks of the foxy effect
    scabbs: i32,                    // number of stacks of the scabbs effect
    next_scabbs: i32,               // number of stacks of scabbs effect after this one
    pub deck: Vec<Card>,            // the cards left in the deck
    pub turn: i32,                  // the current turn
    prep_pending: bool,             // whether we have a preparation effect pending
    pub fish: Vec<Card>,            // the cards we can select for the pending Go Fishin'
}

// Representation of the different ways to play a card
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Play {
    pub index: usize,      // which card in hand to play
    target: Option<usize>, // which card on the board to target, if any
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Action {
    Play(Play),    // playing a card from hand
    Choose(usize), // for selecting a Gone Fishin' card
    EndTurn,
}

#[derive(Clone, Debug)]
pub enum Plan {
    Win(Vec<Play>),
    Lose,
    Timeout,
}

// Return a random index satisfying the predicate, or None if none does
fn random_index_where<T>(v: &Vec<T>, f: impl Fn(&T) -> bool) -> Option<usize> {
    let mut rng = rand::thread_rng();
    match v.iter().enumerate().filter(|(_, x)| f(x)).choose(&mut rng) {
        Some((i, _)) => Some(i),
        None => None,
    }
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
        if self.prep_pending {
            writeln!(f, "prep_pending")?;
        }
        Ok(())
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Vec::new(),
            hand: Vec::new(),
            passage: Vec::new(),
            life: 30,
            mana: 0,
            storm: 0,
            foxy: 0,
            scabbs: 0,
            next_scabbs: 0,
            deck: Vec::new(),
            turn: 0,
            prep_pending: false,
            fish: Vec::new(),
        }
    }

    pub fn hand_string(&self) -> String {
        self.hand
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn play_string(&self, play: &Play) -> String {
        let mut s = self.hand[play.index].to_string();
        if let Some(t) = play.target {
            s.push_str(" -> ");
            s.push_str(&self.board[t].to_string());
        }
        s
    }

    pub fn action_string(&self, action: &Action) -> String {
        match action {
            Action::Play(play) => self.play_string(play),
            Action::Choose(i) => self.fish[*i].to_string(),
            Action::EndTurn => "end turn".to_string(),
        }
    }

    pub fn can_combo(&self) -> bool {
        self.storm > 0
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

    pub fn add_card_instances_to_hand(&mut self, iter: impl Iterator<Item = CardInstance>) {
        for ci in iter {
            if self.hand.len() >= 10 {
                break;
            }
            self.hand.push(ci);
        }
    }

    pub fn add_card_instance_to_hand(&mut self, ci: CardInstance) {
        self.add_card_instances_to_hand(iter::once(ci))
    }

    pub fn add_cards_to_hand(&mut self, iter: impl Iterator<Item = Card>) {
        self.add_card_instances_to_hand(iter.map(|c| CardInstance::new(&c)))
    }

    pub fn add_card_to_hand(&mut self, card: &Card) {
        self.add_card_instance_to_hand(CardInstance::new(card))
    }

    pub fn turn_is_fresh(&self) -> bool {
        self.storm == 0 && self.turn == self.mana
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

    // Draws a random card obeying the given predicate
    // Returns whether we succeeded
    fn draw_from(&mut self, pred: impl Fn(&Card) -> bool) -> bool {
        match random_index_where(&self.deck, |c| pred(c)) {
            Some(i) => {
                let card = self.deck.remove(i);
                self.add_card_to_hand(&card);
                true
            }
            None => false,
        }
    }

    // Draws the first card obeying the given predicate
    // Returns whether we succeeded
    fn draw_first(&mut self, pred: impl Fn(&Card) -> bool) -> bool {
        match self.deck.iter().position(|c| pred(c)) {
            Some(i) => {
                let card = self.deck.remove(i);
                self.add_card_to_hand(&card);
                true
            }
            None => false,
        }
    }

    // Draws one random card into our hand
    fn draw(&mut self) -> bool {
        self.draw_from(|_| true)
    }

    fn draw_minion(&mut self) -> bool {
        self.draw_from(|c| c.minion())
    }

    fn draw_first_minion(&mut self) -> bool {
        self.draw_first(|c| c.minion())
    }

    fn draw_spell(&mut self) -> bool {
        self.draw_from(|c| c.spell())
    }

    fn draw_specific(&mut self, card: &Card) -> bool {
        self.draw_from(|c| c == card)
    }

    pub fn new_going_first(deck: &[Card]) -> Self {
        let mut game = Self::new();
        game.deck = deck.to_vec();
        game.draw();
        game.draw();
        game.draw();
        game.end_turn();
        game
    }

    pub fn new_going_second(deck: &[Card]) -> Self {
        let mut game = Self::new();
        game.deck = deck.to_vec();
        game.draw();
        game.draw();
        game.draw();
        game.draw();
        game.add_card_to_hand(&Card::Coin);
        game.end_turn();
        game
    }

    pub fn new_going_random(deck: &[Card]) -> Self {
        if rand::random() {
            Self::new_going_first(deck)
        } else {
            Self::new_going_second(deck)
        }
    }

    // Play the first card in hand matching the provided card and target
    pub fn play_card(&mut self, card: &Card, target: Option<&Card>) {
        println!("play {} {:?}", card, target);
        let hand_index = self
            .hand
            .iter()
            .position(|c| c.card == *card)
            .expect("card not in hand");
        let move_target = match target {
            Some(t) => Some(
                self.board
                    .iter()
                    .position(|c| *c == *t)
                    .expect("target not on board"),
            ),
            None => None,
        };
        let play = Play {
            index: hand_index,
            target: move_target,
        };

        // Check that m is in the list of possible moves
        if !self.plays().contains(&play) {
            // print the board
            println!("{}", self);
            println!("possible moves: {:?}", self.plays());
            panic!("impossible move: {:?}", play);
        }

        self.play(&play);
    }

    pub fn play(&mut self, play: &Play) {
        let card = self.hand[play.index];
        self.mana -= self.cost(play.index);
        assert!(self.mana >= 0);
        self.hand.remove(play.index);
        self.scabbs = self.next_scabbs;
        self.next_scabbs = 0;

        if card.card == Card::Tenwu {
            let target_index = play.target.unwrap();
            let target_card = self.board[target_index];
            let mut ci = CardInstance::new(&target_card);
            ci.tenwu = true;
            self.add_card_instance_to_hand(ci);
            self.board.remove(target_index);
        }

        if card.card.minion() {
            self.board.push(card.card);
        } else if card.card.spell() {
            self.prep_pending = false;
        }

        if card.card.combo() {
            self.foxy = 0;
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
                let target_card = self.board.remove(play.target.unwrap());
                let mut ci = CardInstance::new(&target_card);
                ci.cost_reduction = 2;
                self.add_card_instance_to_hand(ci);
            }
            Card::Shroud => {
                if self.minions_in_deck() <= 2 {
                    // We want to do this deterministically
                    self.draw_first_minion();
                    self.draw_first_minion();
                } else {
                    self.draw_minion();
                    self.draw_minion();
                }
            }
            Card::Swindle => {
                self.draw_spell();
                if self.storm > 0 {
                    self.draw_minion();
                }
            }
            Card::Door => {
                self.draw_spell();
            }
            Card::Extortion => {
                self.draw();
            }
            Card::Cutlass => {
                self.draw();
                match random_index_where::<CardInstance>(&self.hand, |c| {
                    c.card.spell() && c.cost() > 0
                }) {
                    Some(i) => {
                        self.hand[i].cost_reduction += 1;
                    }
                    None => (),
                }
            }
            Card::GoneFishin => {
                if self.deck.len() <= 3 {
                    self.fish = self.deck.clone();
                } else {
                    let mut rng = &mut rand::thread_rng();
                    self.fish = self.deck.choose_multiple(&mut rng, 3).cloned().collect();
                }
            }
            Card::SecretPassage => {
                self.passage.extend(self.hand.iter());
                self.hand = vec![];
                self.draw();
                self.draw();
                self.draw();
                self.draw();
                for c in &mut self.hand {
                    c.passage = true;
                }
            }
            _ => (),
        }

        if !card.card.is_trade() {
            self.storm += 1;
        }
    }

    pub fn take_action(&mut self, action: &Action) {
        match action {
            Action::Play(m) => self.play(m),
            Action::Choose(i) => {
                let card = self.fish[*i];
                self.draw_specific(&card);
                self.fish.clear();
            }
            Action::EndTurn => self.end_turn(),
        }
    }

    pub fn can_end_turn(&self) -> bool {
        self.fish.is_empty()
    }

    // Ends turn and starts the next one
    pub fn end_turn(&mut self) {
        assert!(self.can_end_turn());
        for c in &mut self.hand {
            c.tenwu = false;
        }

        // This might not put the cards in the right order when we play
        // Secret Passage multiple times
        let mut new_hand: Vec<CardInstance> = vec![];
        for ci in self.hand.iter().chain(self.passage.iter()) {
            if ci.passage {
                self.deck.push(ci.card);
            } else {
                new_hand.push(*ci);
            }
        }
        self.hand = new_hand;
        self.passage = vec![];

        self.foxy = 0;
        self.scabbs = 0;
        self.next_scabbs = 0;
        self.prep_pending = false;
        self.storm = 0;
        self.turn += 1;
        self.mana = std::cmp::min(10, self.turn);

        self.draw();
    }

    pub fn actions(&self) -> Vec<Action> {
        if !self.fish.is_empty() {
            // Return a move for each index in fish
            return (0..self.fish.len()).map(|i| Action::Choose(i)).collect();
        }
        let mut answer = vec![Action::EndTurn];
        for (index, ci) in self.hand.iter().enumerate() {
            if !self.can_play(index) {
                continue;
            }
            if ci.card.must_target() {
                for target in 0..self.board.len() {
                    answer.push(Action::Play(Play {
                        index,
                        target: Some(target),
                    }));
                }
            } else {
                answer.push(Action::Play(Play {
                    index,
                    target: None,
                }))
            }
        }
        answer
    }

    pub fn plays(&self) -> Vec<Play> {
        self.actions()
            .into_iter()
            .filter_map(|action| match action {
                Action::Play(p) => Some(p),
                _ => None,
            })
            .collect()
    }

    // A heuristic for which moves we should consider if there is no deterministic kill
    pub fn non_kill_actions(&self) -> Vec<Action> {
        self.actions()
            .into_iter()
            .filter(|action| match action {
                Action::Play(p) => {
                    let card = self.hand[p.index].card;
                    !card.minion() && card != Card::Shadowstep
                }
                _ => true,
            })
            .collect()
    }

    fn minions_in_deck(&self) -> usize {
        self.deck.iter().filter(|c| c.minion()).count()
    }

    // A heuristic for which moves we should consider when searching for a deterministic kill
    pub fn deterministic_plays(&self) -> Vec<Play> {
        self.actions()
            .into_iter()
            .filter_map(|action| match action {
                Action::Play(p) => match self.hand[p.index].card {
                    Card::GoneFishin => None,
                    Card::SecretPassage => None,
                    Card::Swindle => None,
                    Card::Door => None,
                    Card::Cutlass => None,
                    Card::Extortion => None,
                    Card::Shroud => {
                        if self.minions_in_deck() <= 2 {
                            Some(p)
                        } else {
                            None
                        }
                    }
                    _ => Some(p),
                },
                _ => None,
            })
            .collect()
    }

    fn is_win(&self) -> bool {
        self.life <= 0
    }

    pub fn hash_value(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    // Returns a plan with reversed moves.
    // cache contains a map from hash of a game state to a plan for it, also with reversed moves.
    // We update cache as we go.
    fn find_deterministic_win_helper(
        &self,
        start: Instant,
        time_limit: f64,
        cache: &mut HashMap<u64, Plan>,
    ) -> Plan {
        if start.elapsed().as_secs() as f64 > time_limit {
            return Plan::Timeout;
        }
        if self.is_win() {
            return Plan::Win(Vec::new());
        }

        // Check if we already have a plan for this game state
        let hash = self.hash_value();
        if let Some(plan) = cache.get(&hash) {
            return plan.clone();
        }

        for play in self.deterministic_plays() {
            let mut clone = self.clone();
            clone.play(&play);
            match clone.find_deterministic_win_helper(start, time_limit, cache) {
                Plan::Win(mut plays) => {
                    plays.push(play);
                    let plan = Plan::Win(plays);
                    cache.insert(hash, plan.clone());
                    return plan;
                }
                Plan::Lose => (),
                Plan::Timeout => return Plan::Timeout,
            }
        }

        // Our search is exhausted
        cache.insert(hash, Plan::Lose);
        Plan::Lose
    }

    // Returns a plan with list of moves to win.
    pub fn find_deterministic_win(&self, time_limit: f64) -> Plan {
        let start = Instant::now();
        let mut cache = HashMap::new();
        match self.find_deterministic_win_helper(start, time_limit, &mut cache) {
            Plan::Win(mut plays) => {
                plays.reverse();
                Plan::Win(plays)
            }
            x => x,
        }
    }

    // Returns whether we won or not.
    pub fn print_deterministic_win(&self, time_limit: f64) -> bool {
        let plan = self.find_deterministic_win(time_limit);
        match plan {
            Plan::Win(plays) => {
                println!("win found:");
                let mut clone = self.clone();
                for play in plays {
                    println!("{}", clone.play_string(&play));
                    clone.play(&play);
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

// Expects that a win can be found with these parameters but not one more life
pub fn assert_exact_win_with_deck(mana: i32, life: i32, hand: Vec<Card>, deck: Vec<Card>) {
    let mut game = Game::new();
    game.mana = mana;
    game.life = life;
    game.add_cards_to_hand(hand.into_iter());
    game.deck = deck;
    assert_matches!(game.find_deterministic_win(1.0), Plan::Win(_));
    game.life += 1;
    match game.find_deterministic_win(1.0) {
        Plan::Win(plays) => {
            println!("game: {}", game);
            for play in plays {
                println!("{}", game.play_string(&play));
                game.play(&play);
                println!("mana: {}, life: {}", game.mana, game.life);
            }
            panic!("expected no win");
        }
        Plan::Lose => (),
        Plan::Timeout => panic!("timeout in find_win"),
    }
}

pub fn assert_exact_win(mana: i32, life: i32, hand: Vec<Card>) {
    assert_exact_win_with_deck(mana, life, hand, Vec::new());
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

    #[test]
    fn basic_foxy_win() {
        let mut g: Game = Game::new();
        g.mana = 4;
        g.life = 30;
        let hand = vec![
            Card::Foxy,
            Card::Shadowstep,
            Card::Scabbs,
            Card::Shark,
            Card::Tenwu,
            Card::Pillager,
            Card::Pillager,
        ];
        g.add_cards_to_hand(hand.into_iter());
        g.play_card(&Card::Foxy, None);
        g.play_card(&Card::Shadowstep, Some(&Card::Foxy));
        g.play_card(&Card::Foxy, None);
        g.play_card(&Card::Scabbs, None);
        g.play_card(&Card::Shark, None);
        g.play_card(&Card::Tenwu, Some(&Card::Scabbs));
        g.play_card(&Card::Scabbs, None);
        g.play_card(&Card::Pillager, None);
        g.play_card(&Card::Pillager, None);
        assert!(g.life <= 0);
    }

    #[test]
    fn anti_renathal_win() {
        let mut g: Game = Game::new();
        g.mana = 7;
        g.life = 44;
        let hand = vec![
            Card::Foxy,
            Card::Shadowstep,
            Card::Scabbs,
            Card::Shark,
            Card::Tenwu,
            Card::Pillager,
            Card::Pillager,
        ];
        g.add_cards_to_hand(hand.into_iter());
        g.play_card(&Card::Foxy, None);
        g.play_card(&Card::Scabbs, None);
        g.play_card(&Card::Shark, None);
        g.play_card(&Card::Tenwu, Some(&Card::Scabbs));
        g.play_card(&Card::Shadowstep, Some(&Card::Tenwu));
        g.play_card(&Card::Scabbs, None);
        g.play_card(&Card::Pillager, None);
        g.play_card(&Card::Pillager, None);
        g.play_card(&Card::Tenwu, Some(&Card::Pillager));
        g.play_card(&Card::Pillager, None);
        assert!(g.life <= 0);
    }

    // Keep these tests sorted by mana, then life

    #[test]
    fn t3_kill() {
        assert_exact_win(
            3,
            34,
            vec![
                Card::Coin,
                Card::Foxy,
                Card::Shadowstep,
                Card::Scabbs,
                Card::Shark,
                Card::Tenwu,
                Card::Pillager,
                Card::Pillager,
            ],
        )
    }

    #[test]
    fn find_basic_foxy_win() {
        assert_exact_win(
            4,
            30,
            vec![
                Card::Foxy,
                Card::Shadowstep,
                Card::Scabbs,
                Card::Shark,
                Card::Tenwu,
                Card::Pillager,
                Card::Pillager,
            ],
        )
    }

    #[test]
    fn basic_dancer() {
        assert_exact_win(
            4,
            34,
            vec![
                Card::Coin,
                Card::Dancer,
                Card::Shadowstep,
                Card::Scabbs,
                Card::Shark,
                Card::Pillager,
                Card::Pillager,
            ],
        )
    }

    #[test]
    fn potion_and_two_pillagers() {
        assert_exact_win(
            4,
            54,
            vec![
                Card::Coin,
                Card::Dancer,
                Card::Shadowstep,
                Card::Potion,
                Card::Scabbs,
                Card::Shark,
                Card::Pillager,
                Card::Pillager,
            ],
        )
    }

    #[test]
    fn potion_and_tenwu() {
        assert_exact_win(
            4,
            62,
            vec![
                Card::Coin,
                Card::Dancer,
                Card::Shadowstep,
                Card::Potion,
                Card::Scabbs,
                Card::Shark,
                Card::Pillager,
                Card::Tenwu,
            ],
        )
    }

    #[test]
    fn shark_missing() {
        assert_exact_win(
            5,
            28,
            vec![
                Card::Coin,
                Card::Dancer,
                Card::Shadowstep,
                Card::Scabbs,
                Card::Tenwu,
                Card::Pillager,
                Card::Potion,
            ],
        )
    }

    #[test]
    fn basic_foxy_analog() {
        assert_exact_win(
            5,
            34,
            vec![
                Card::Coin,
                Card::Dancer,
                Card::Scabbs,
                Card::Shark,
                Card::Tenwu,
                Card::Pillager,
                Card::Pillager,
            ],
        )
    }

    #[test]
    fn pillager_missing_with_foxy() {
        assert_exact_win(
            5,
            36,
            vec![
                Card::Foxy,
                Card::Shadowstep,
                Card::Shadowstep,
                Card::Scabbs,
                Card::Shark,
                Card::Tenwu,
                Card::Pillager,
            ],
        )
    }

    #[test]
    fn pillager_missing_with_dancer() {
        assert_exact_win(
            5,
            36,
            vec![
                Card::Coin,
                Card::Shadowstep,
                Card::Tenwu,
                Card::Scabbs,
                Card::Shark,
                Card::Dancer,
                Card::Pillager,
            ],
        )
    }

    #[test]
    fn free_card() {
        assert_exact_win(
            5,
            46,
            vec![
                Card::Coin,
                Card::Cloak,
                Card::Shadowstep,
                Card::Potion,
                Card::Scabbs,
                Card::Shark,
                Card::Dancer,
                Card::Pillager,
            ],
        )
    }

    #[test]
    fn using_shroud() {
        assert_exact_win_with_deck(
            5,
            50,
            vec![
                Card::Coin,
                Card::Shroud,
                Card::Shadowstep,
                Card::Scabbs,
                Card::Dancer,
                Card::Tenwu,
                Card::Pillager,
            ],
            vec![Card::Shark, Card::Pillager, Card::Coin],
        )
    }

    #[test]
    fn fox_scabbs_core() {
        assert_exact_win(
            6,
            22,
            vec![
                Card::Foxy,
                Card::Scabbs,
                Card::Shark,
                Card::Tenwu,
                Card::Pillager,
                Card::Pillager,
            ],
        )
    }

    #[test]
    fn advanced_foxy_analog() {
        assert_exact_win(
            6,
            62,
            vec![
                Card::Coin,
                Card::Dancer,
                Card::Scabbs,
                Card::Shadowstep,
                Card::Shark,
                Card::Tenwu,
                Card::Pillager,
                Card::Pillager,
            ],
        )
    }

    #[test]
    fn find_anti_renathal_win() {
        assert_exact_win(
            7,
            44,
            vec![
                Card::Foxy,
                Card::Shadowstep,
                Card::Scabbs,
                Card::Shark,
                Card::Tenwu,
                Card::Pillager,
                Card::Pillager,
            ],
        )
    }

    #[test]
    fn find_druid_line() {
        assert_exact_win(
            8,
            72,
            vec![
                Card::Foxy,
                Card::Shadowstep,
                Card::Shadowstep,
                Card::Scabbs,
                Card::Shark,
                Card::Tenwu,
                Card::Pillager,
                Card::Pillager,
            ],
        )
    }
}
