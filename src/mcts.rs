use std::collections::HashMap;
use std::iter::zip;

use crate::{
    game::{Game, Move, Plan},
    player::escape_bot_play,
};

// A policy gives a distribution among possible moves for a given game state
type Policy = fn(&Game, &Vec<Option<Move>>) -> Vec<f32>;

#[derive(Clone, Debug)]
struct StateActionData {
    action: Option<Move>,

    // The probability distribution from our shallow policy
    // Also known as P(s, a)
    shallow: f32,

    // Average reward of this (state, action) pair in playouts
    // Also known as Q(s, a)
    reward: f32,

    // Number of times this (state, action) pair has been visited
    // Also known as N(s, a)
    visits: u32,
}

// Information relevant to a game state during the MCTS playout
// The vectors are parallel to non_kill_candidate_moves
#[derive(Clone, Debug)]
struct StateData {
    deterministic_win: bool,
    actions: Vec<StateActionData>,
}

impl StateData {
    fn new(game: &Game, policy: Policy) -> StateData {
        let actions = game.non_kill_candidate_moves();
        let shallow = policy(&game, &actions);
        StateData {
            deterministic_win: false,
            actions: zip(actions, shallow)
                .map(|(action, shallow)| StateActionData {
                    action,
                    shallow,
                    reward: 0.0,
                    visits: 0,
                })
                .collect(),
        }
    }

    fn new_win() -> StateData {
        StateData {
            deterministic_win: true,
            actions: Vec::new(),
        }
    }

    // Pick the index with the highest upper confidence bound
    fn explore_index(&self) -> usize {
        let total_visits = self.actions.iter().map(|a| a.visits).sum::<u32>() as f32;

        // Give each candidate an upper confidence bound on the expected value of the reward
        // Also known as U(s, a).
        // See U(s, a) formula from:
        //   https://web.stanford.edu/~surag/posts/alphazero.html
        // We add the 0.01 so that we get something reasonable when the Q(s, a) are all zero
        let exploration_parameter = 1.0;
        let numerator = (0.01 + total_visits as f32).sqrt() * exploration_parameter;
        let upper_bounds: Vec<f32> = self
            .actions
            .iter()
            .map(|a| a.reward + numerator * a.shallow / (1.0 + a.visits as f32))
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
        let mut action = &mut self.actions[index];
        action.reward =
            (action.reward * action.visits as f32 + reward) / (action.visits as f32 + 1.0);
        action.visits += 1;
    }

    // Pick the best reward, ignoring confidence
    fn best_move(&self) -> Option<Move> {
        self.actions
            .iter()
            .max_by(|a, b| a.reward.total_cmp(&b.reward))
            .unwrap()
            .action
    }
}

pub struct MCTS {
    // Data for state-action pairs
    state_map: HashMap<Game, StateData>,

    policy: Policy,
}

// Design the reward to be nonnegative so that it looks better than branches we haven't tried
const MAX_TURNS: i32 = 10;
fn reward(game: &Game) -> f32 {
    (MAX_TURNS - game.turn) as f32
}

impl MCTS {
    pub fn new(policy: Policy) -> MCTS {
        MCTS {
            state_map: HashMap::new(),
            policy,
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
            None => StateData::new(game, self.policy),
        };

        // Choose a move
        let i = state_data.explore_index();
        let mut game_clone = game.clone();
        match state_data.actions[i].action {
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

pub fn random_policy(_: &Game, actions: &Vec<Option<Move>>) -> Vec<f32> {
    (0..actions.len())
        .map(|_| 1.0 / actions.len() as f32)
        .collect()
}

pub fn escape_policy(game: &Game, actions: &Vec<Option<Move>>) -> Vec<f32> {
    let m = escape_bot_play(game);
    // Find m in actions
    match actions.iter().position(|a| a == &m) {
        Some(i) => (0..actions.len())
            .map(|j| {
                if i == j {
                    0.6
                } else {
                    0.4 / (actions.len() - 1) as f32
                }
            })
            .collect(),
        None => random_policy(game, actions),
    }
}

pub fn mcts_play(game: &Game) -> Option<Move> {
    let mut mcts = MCTS::new(random_policy);
    for _ in 0..50 {
        mcts.playout(game);
    }
    mcts.best_move(game)
}
