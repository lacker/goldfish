use std::collections::BTreeMap;

use goldfish::card::PANDA_DECK;
use goldfish::game::Game;
use goldfish::mcts::mcts_play;

const NUM_GAMES: usize = 100;

fn main() {
    println!("evaluating...");

    // turn_map maps the turn to the number of games where we won on that turn
    let mut turn_map = BTreeMap::new();

    for i in 0..NUM_GAMES {
        let mut game = Game::new_going_random(PANDA_DECK);

        loop {
            if game.print_deterministic_win(1.0) {
                println!("game {} won on turn {}", i, game.turn);
                break;
            }

            while let Some(m) = mcts_play(&game) {
                game.make_move(&m);
            }

            if game.turn >= 10 {
                println!("game {} failed", i);
                break;
            }

            game.end_turn();
        }
        *turn_map.entry(game.turn).or_insert(0) += 1;
        println!();
    }

    println!("results:");
    let mut sum = 0;
    // Iterate over turn map, sorted by key
    for (turn, num_wins) in turn_map {
        sum += turn * num_wins;
        println!(
            "turn {}{}: {} wins",
            turn,
            if turn == 10 { "+" } else { "" },
            num_wins
        );
    }
    println!("average win turn: {}", sum as f64 / NUM_GAMES as f64);
}
