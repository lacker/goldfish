use std::collections::HashMap;
use std::iter::zip;

use crate::game::{Game, Move, Plan};

// Information relevant to a game state during the MCTS playout
// The vectors are parallel to non_kill_candidate_moves
#[derive(Clone, Debug)]
struct StateData {
    deterministic_win: bool,

    // The possible actions
    actions: Vec<Option<Move>>,

    // Average reward of this (state, action) pair in playouts
    rewards: Vec<f32>,

    // Number of times this (state, action) pair has been visited
    visits: Vec<u32>,
}

impl StateData {
    fn new(game: &Game) -> StateData {
        let mut actions = Vec::new();
        let mut rewards = Vec::new();
        let mut visits = Vec::new();
        for m in game.non_kill_candidate_moves() {
            actions.push(m);
            rewards.push(0.0);
            visits.push(0);
        }
        StateData {
            deterministic_win: false,
            actions,
            rewards,
            visits,
        }
    }

    fn new_win() -> StateData {
        StateData {
            deterministic_win: true,
            actions: Vec::new(),
            rewards: Vec::new(),
            visits: Vec::new(),
        }
    }

    fn total_visits(&self) -> u32 {
        self.visits.iter().sum()
    }

    // Pick the index with the highest upper confidence bound
    fn explore_index(&self) -> usize {
        // Treating P(s, a) as an even distribution
        let confidence_term = (self.total_visits() as f32).sqrt() / (self.actions.len() as f32);

        // Give each candidate an upper confidence bound on the expected value
        let upper_bounds: Vec<f32> = zip(self.rewards.iter(), self.visits.iter())
            .map(|(reward, visits)| reward + confidence_term / (1.0 + *visits as f32))
            .collect();

        // Pick the candidate with the highest upper confidence bound
        upper_bounds
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .unwrap()
            .0
    }

    fn update(&mut self, index: usize, reward: f32) {
        self.rewards[index] = (self.rewards[index] * self.visits[index] as f32 + reward)
            / (self.visits[index] as f32 + 1.0);
        self.visits[index] += 1;
    }

    // Pick the best reward, ignoring confidence
    fn best_move(&self) -> Option<Move> {
        let best_index = self
            .rewards
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .unwrap()
            .0;
        self.actions[best_index]
    }
}

pub struct MCTS {
    // Data for state-action pairs
    state_map: HashMap<Game, StateData>,
}

// Currently the reward is the "negative winning turn", capped at -10
const MAX_TURNS: i32 = 10;
fn reward(game: &Game) -> f32 {
    -game.turn as f32
}

impl MCTS {
    pub fn new() -> MCTS {
        MCTS {
            state_map: HashMap::new(),
        }
    }

    // Does a playout from the provided game state
    // Returns the reward for the playout.
    pub fn playout(&mut self, game: &Game) -> f32 {
        if game.turn >= MAX_TURNS {
            return reward(game);
        }

        let state_data = self.state_map.get(&game);
        if state_data.is_none() && game.storm == 0 {
            // Check for a deterministic win
            if let Plan::Win(_) = game.find_deterministic_win(0.1) {
                let answer = reward(game);
                self.state_map.insert(game.clone(), StateData::new_win());
                return answer;
            }
        }
        if let Some(state_data) = state_data {
            if state_data.deterministic_win {
                // We already have found that this is a deterministic win
                return reward(game);
            }
        }

        let mut state_data = match state_data {
            Some(s) => s.clone(),
            None => StateData::new(game),
        };

        // Choose a move
        let i = state_data.explore_index();
        let mut game_clone = game.clone();
        match state_data.actions[i] {
            Some(m) => game_clone.make_move(&m),
            None => game_clone.end_turn(),
        }

        // Recurse
        let answer = self.playout(&game_clone);

        // Update with the results of the playout
        state_data.update(i, answer);
        if let Some(s) = self.state_map.get_mut(game) {
            *s = state_data;
        } else {
            self.state_map.insert(game.clone(), state_data);
        }

        answer
    }

    // Returns the best move.
    // If we have no idea, just pick the first valid move.
    pub fn best_move(&self, game: &Game) -> Option<Move> {
        match self.state_map.get(game) {
            Some(s) => s.best_move(),
            None => game.non_kill_candidate_moves()[0],
        }
    }
}

pub fn mcts_play(game: &Game) -> Option<Move> {
    let mut mcts = MCTS::new();
    for _ in 0..50 {
        mcts.playout(game);
    }
    mcts.best_move(game)
}
