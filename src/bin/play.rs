use goldfish::card::PANDA_DECK;
use goldfish::game::Game;
use goldfish::mcts::mcts_action;
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
        if game.turn_is_fresh() {
            println!("\nturn {}", game.turn);
            println!("{}", game);
            if game.print_deterministic_win(5.0) {
                break;
            }
        }

        let action = mcts_action(&game);
        println!("\naction: {}", game.action_string(&action));
        game.take_action(&action);

        println!("hand: {}", game.hand_string());
        println!("mana: {}", game.mana);

        if game.turn >= 10 {
            println!("we give up");
            break;
        }
    }
}
