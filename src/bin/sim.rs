use goldfish::game::Game;
use goldfish::game::Plan;

fn main() {
    let mut game = Game::new_going_first();
    while game.turn < 10 {
        game.next_turn();
        println!("turn {}. hand = {}", game.turn, game.hand_string());
        let plan = game.find_win();
        match plan {
            Plan::Win(moves) => {
                println!("win found:");
                for m in moves {
                    println!("{}", game.move_string(&m));
                    game.make_move(&m);
                }
                return;
            }
            Plan::Lose => {
                println!("cannot win");
            }
            Plan::Timeout => {
                println!("no win found");
            }
        }
        println!();
    }
}
