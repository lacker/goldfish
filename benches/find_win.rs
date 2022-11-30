#[macro_use]
extern crate assert_matches;

use goldfish::card::Card;
use goldfish::game::{Game, Plan};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Expects that a win can be found with these parameters
fn assert_win(mana: i32, life: i32, hand: Vec<Card>) {
    let mut game = Game::new();
    game.mana = black_box(mana);
    game.life = life;
    game.add_cards_to_hand(hand.into_iter());
    assert_matches!(game.find_win(), Plan::Win(_));
}

// Examples from:
// https://www.reddit.com/r/CompetitiveHS/comments/wq6snr/wild_guide_to_six_minion_pillager_rogue/
pub fn basic_foxy(c: &mut Criterion) {
    c.bench_function("most basic foxy", |b| {
        b.iter(|| {
            assert_win(
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
        })
    });
}

criterion_group!(benches, basic_foxy);
criterion_main!(benches);
