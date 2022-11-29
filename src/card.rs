use enum_iterator::Sequence;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt;

pub const UNKNOWN_COST: i32 = 20;

// All the cards we handle.
// Sort by roughly the order that you expect to play cards, to help win search.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Sequence)]
pub enum Card {
    Coin,
    Shark,
    Dancer,
    Foxy,
    Scabbs,
    Pillager,
    Potion,
    Shadowstep,
    Tenwu,
    GoneFishin,
    Shroud,
    SecretPassage,
    Swindle,
    Evasion,
    Door,
    Cutlass,
    Extortion,
    Preparation,
    BoneSpike,
    Unknown,
}

// Only the good cards, taken from:
// https://www.vicioussyndicate.com/decks/pillager-rogue-4/
pub const STARTING_DECK: &'static [Card] = &[
    Card::Coin,
    Card::Coin,
    Card::Shadowstep,
    Card::Shadowstep,
    Card::Foxy,
    Card::Dancer,
    Card::Tenwu,
    Card::Potion,
    Card::Scabbs,
    Card::Shark,
    Card::Pillager,
    Card::Pillager,
];

lazy_static! {
    static ref CARD_FOR_NAME: HashMap<String, Card> = {
        let mut m = HashMap::new();
        for card in enum_iterator::all::<Card>() {
            m.insert(card.to_string(), card);
        }
        m
    };
}

// Short form of the card name
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Card::BoneSpike => "Serrated Bone Spike",
            Card::Coin => "The Coin",
            Card::Cutlass => "Blackwater Cutlass",
            Card::Dancer => "Mailbox Dancer",
            Card::Door => "Door of Shadows",
            Card::Evasion => "Evasion",
            Card::Extortion => "SI:7 Extortion",
            Card::Foxy => "Foxy Fraud",
            Card::GoneFishin => "Gone Fishin'",
            Card::Pillager => "Spectral Pillager",
            Card::Potion => "Potion of Illusion",
            Card::Preparation => "Preparation",
            Card::Scabbs => "Scabbs Cutterbutter",
            Card::SecretPassage => "Secret Passage",
            Card::Shadowstep => "Shadowstep",
            Card::Shark => "Spirit of the Shark",
            Card::Shroud => "Shroud of Concealment",
            Card::Swindle => "Swindle",
            Card::Tenwu => "Tenwu of the Red Smoke",
            Card::Unknown => "Unknown",
        })
    }
}

impl Card {
    // Must match the log file output
    pub fn from_name(s: &str) -> Self {
        if let Some(card) = CARD_FOR_NAME.get(s) {
            return *card;
        }
        match s {
            "Bananas" => Card::Unknown,
            "Counterfeit Coin" => Card::Coin,
            _ => {
                println!("unknown card name: {}", s);
                Card::Unknown
            }
        }
    }

    pub fn from_card_id(card_id: &str) -> Self {
        match card_id {
            "REV_939" => Card::BoneSpike,
            "GAME_005" => Card::Coin,
            "CFM_630" => Card::Coin, // for Counterfeit Coin
            "DED_004" => Card::Cutlass,
            "SW_070" => Card::Dancer,
            "REV_938" => Card::Door,
            "LOOT_214" => Card::Evasion,
            "SW_412" => Card::Extortion,
            "DMF_511" => Card::Foxy,
            "TSC_916" => Card::GoneFishin,
            "ICC_910" => Card::Pillager,
            "SCH_352" => Card::Potion,
            "CORE_EX1_145" => Card::Preparation,
            "BAR_552" => Card::Scabbs,
            "SCH_305" => Card::SecretPassage,
            "CORE_EX1_144" => Card::Shadowstep,
            "WC_016" => Card::Shroud,
            "TRL_092" => Card::Shark,
            "DMF_515" => Card::Swindle,
            "DMF_071" => Card::Tenwu,
            _ => {
                // println!("unknown card id: {}", card_id);
                Card::Unknown
            }
        }
    }

    pub fn cost(&self) -> i32 {
        match self {
            Card::Coin => 0,
            Card::Shadowstep => 0,
            Card::Dancer => 2,
            Card::Foxy => 2,
            Card::Pillager => 6,
            Card::Potion => 4,
            Card::Scabbs => 4,
            Card::Shark => 4,
            Card::Tenwu => 2,
            _ => UNKNOWN_COST, // Just forbid casting unimplemented cards
        }
    }

    pub fn minion(&self) -> bool {
        match self {
            Card::Dancer => true,
            Card::Foxy => true,
            Card::Pillager => true,
            Card::Scabbs => true,
            Card::Shark => true,
            Card::Tenwu => true,
            _ => false,
        }
    }

    pub fn spell(&self) -> bool {
        match self {
            Card::BoneSpike => true,
            Card::Coin => true,
            Card::Door => true,
            Card::Evasion => true,
            Card::Extortion => true,
            Card::GoneFishin => true,
            Card::Potion => true,
            Card::Preparation => true,
            Card::SecretPassage => true,
            Card::Shadowstep => true,
            Card::Shroud => true,
            Card::Swindle => true,
            _ => false,
        }
    }

    pub fn weapon(&self) -> bool {
        match self {
            Card::Cutlass => true,
            _ => false,
        }
    }

    pub fn combo(&self) -> bool {
        match self {
            Card::Pillager => true,
            Card::Scabbs => true,
            _ => false,
        }
    }

    pub fn must_target(&self) -> bool {
        match self {
            Card::Shadowstep => true,
            Card::Tenwu => true,
            _ => false,
        }
    }
}

// Properties that apply to only the specific version of this card, in our hand.
// This could extend to on-board properties later.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct CardInstance {
    pub card: Card,
    pub potion: bool,
    pub tenwu: bool,
    pub cost_reduction: i32,
}

impl fmt::Display for CardInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.card.to_string()).unwrap();
        if self.potion {
            f.write_str(" (potion)").unwrap();
        }
        if self.tenwu {
            f.write_str(" (tenwu)").unwrap();
        }
        if self.cost_reduction > 0 {
            f.write_str(&format!(" (-{})", self.cost_reduction))
                .unwrap();
        }
        Ok(())
    }
}

impl CardInstance {
    pub fn new(card: &Card) -> Self {
        Self {
            card: *card,
            potion: false,
            tenwu: false,
            cost_reduction: 0,
        }
    }

    pub fn cost(&self) -> i32 {
        let base = if self.potion || self.tenwu {
            1
        } else {
            self.card.cost()
        };
        std::cmp::max(0, base - self.cost_reduction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_type_per_card() {
        for card in enum_iterator::all::<Card>() {
            let mut types = 0;
            if card.minion() {
                types += 1;
            }
            if card.spell() {
                types += 1;
            }
            if card.weapon() {
                types += 1;
            }
            if card == Card::Unknown {
                types += 1;
            }
            assert_eq!(types, 1, "card {} has bad types", card);
        }
    }
}
