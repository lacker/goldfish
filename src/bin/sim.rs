use goldfish::game::Game;

fn main() {
    let mut game = Game::new_going_first();
    while game.turn < 10 {
        game.next_turn();
        println!("turn {}. hand = {}", game.turn, game.hand_string());
        game.print_plan();
        println!();
    }
}
