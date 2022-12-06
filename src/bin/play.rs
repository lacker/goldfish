use goldfish::card::PANDA_DECK;
use goldfish::game::Game;
use goldfish::mcts::mcts_play;
use rand::Rng;

fn main() {
    let mut game = if rand::thread_rng().gen_bool(0.5) {
        println!("going first.");
        Game::new_going_first(PANDA_DECK)
    } else {
        println!("going second.");
        Game::new_going_second(PANDA_DECK)
    };

    loop {
        println!("\nturn {}", game.turn);
        println!("{}", game);
        if game.print_deterministic_win(5.0) {
            break;
        }

        while let Some(m) = mcts_play(&game) {
            println!("\nplay {}", game.move_string(&m));
            game.make_move(&m);
            println!("hand: {}", game.hand_string());
            println!("mana: {}", game.mana);
        }

        game.end_turn();

        if game.turn >= 10 {
            println!("we give up");
            break;
        }
    }
}
