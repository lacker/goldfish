use std::fmt;

// Properties that apply to a card wherever it is
// Sort by roughly the order that you expect to play cards, to help win search.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
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
    Brick,
}

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

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Card::Brick => "Brick",
            Card::Coin => "Coin",
            Card::Dancer => "Dancer",
            Card::Foxy => "Foxy",
            Card::Pillager => "Pillager",
            Card::Potion => "Potion",
            Card::Scabbs => "Scabbs",
            Card::Shadowstep => "Shadowstep",
            Card::Shark => "Shark",
            Card::Tenwu => "Tenwu",
        })
    }
}

impl Card {
    pub fn from_name(s: &str) -> Self {
        match s {
            "The Coin" => Card::Coin,
            "Mailbox Dancer" => Card::Dancer,
            "Foxy Fraud" => Card::Foxy,
            "Spectral Pillager" => Card::Pillager,
            "Potion of Illusion" => Card::Potion,
            "Scabbs Cutterbutter" => Card::Scabbs,
            "Shadowstep" => Card::Shadowstep,
            "Spirit of the Shark" => Card::Shark,
            "Tenwu of the Red Smoke" => Card::Tenwu,
            _ => Card::Brick,
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
            Card::Brick => 20,
        }
    }

    pub fn minion(&self) -> bool {
        match self {
            Card::Dancer => true,
            Card::Foxy => true,
            Card::Pillager => true,
            Card::Scabbs => true,
            Card::Shark => true,
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
