use crate::card::Card;
use crate::game::{Game, Move};
use rand::Rng;

// EscapeBot plays according to some shallow, hand-coded heuristics
pub fn escape_bot_play(game: &Game) -> Option<Move> {
    let possible_moves = game.possible_moves();
    if !game.fish.is_empty() {
        // Pick a random move
        let index = rand::thread_rng().gen_range(0..possible_moves.len());
        return Some(possible_moves[index]);
    }

    if game.storm > 0 {
        // Try to play any useful combo cards we have
        for m in &possible_moves {
            let ci = game.hand[m.index];
            match ci.card {
                Card::Swindle => return Some(*m),
                Card::GoneFishin => return Some(*m),
                _ => (),
            }
        }
    }

    // Try to play any useful non-combo cards we have
    for m in possible_moves {
        let ci = game.hand[m.index];
        match ci.card {
            Card::BoneSpike => return Some(m),
            Card::Cloak => return Some(m),
            Card::Coin if ci.passage => return Some(m),
            Card::Cutlass => return Some(m),
            Card::Door => return Some(m),
            Card::Evasion => return Some(m),
            Card::Extortion => return Some(m),
            Card::Preparation => return Some(m),
            Card::SecretPassage => return Some(m),
            Card::Shroud => return Some(m),
            _ => (),
        }
    }

    // Just pass
    None
}
