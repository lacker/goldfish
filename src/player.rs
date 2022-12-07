use crate::card::Card;
use crate::game::{Action, Game};
use rand::seq::SliceRandom;

// EscapeBot plays according to some shallow, hand-coded heuristics
pub fn escape_bot_action(game: &Game) -> Action {
    if !game.fish.is_empty() {
        return *game
            .non_kill_actions()
            .choose(&mut rand::thread_rng())
            .unwrap();
    }

    let plays = game.plays();

    if game.storm > 0 {
        // Try to play any useful combo cards we have
        for play in &plays {
            let ci = game.hand[play.index];
            let useful = match ci.card {
                Card::Swindle => true,
                Card::GoneFishin => true,
                _ => false,
            };
            if useful {
                return Action::Play(*play);
            }
        }
    }

    // Try to play any useful non-combo cards we have
    for play in plays {
        let ci = game.hand[play.index];
        let useful = match ci.card {
            Card::BoneSpike => true,
            Card::Cloak => true,
            Card::Coin if ci.passage => true,
            Card::Cutlass => true,
            Card::Door => true,
            Card::Evasion => true,
            Card::Extortion => true,
            Card::Preparation => true,
            Card::SecretPassage => true,
            Card::Shroud => true,
            _ => false,
        };
        if useful {
            return Action::Play(play);
        }
    }

    // Just pass
    Action::EndTurn
}
