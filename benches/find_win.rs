use goldfish::card::Card;
use goldfish::game::assert_win;

use criterion::{criterion_group, criterion_main, Criterion};

// Examples from:
// https://www.reddit.com/r/CompetitiveHS/comments/wq6snr/wild_guide_to_six_minion_pillager_rogue/
pub fn hard_foxy_examples(c: &mut Criterion) {
    c.bench_function("todo", |b| {
        b.iter(|| {
            assert_win(
                6,
                9,
                vec![
                    Card::Preparation,
                    Card::Preparation,
                    Card::Preparation,
                    Card::Preparation,
                    Card::Preparation,
                    Card::Preparation,
                    Card::Preparation,
                    Card::Preparation,
                    Card::Preparation,
                    Card::Pillager,
                ],
            );
        })
    });
}

criterion_group!(benches, hard_foxy_examples);
criterion_main!(benches);
