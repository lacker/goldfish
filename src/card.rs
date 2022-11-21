use std::fmt;

// Properties that apply to a card wherever it is
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Card {
    Coin,
    Dancer,
    Foxy,
    Pillager,
    Potion,
    Scabbs,
    Shark,
}

// https://www.vicioussyndicate.com/decks/pillager-rogue-4/
const BASE_DECK: &'static [Card] = &[
    Card::Coin,
    Card::Coin,
    Card::Foxy,
    Card::Dancer,
    Card::Potion,
    Card::Scabbs,
    Card::Shark,
    Card::Pillager,
    Card::Pillager,
];

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Card::Coin => "Coin",
            Card::Dancer => "Dancer",
            Card::Foxy => "Foxy",
            Card::Pillager => "Pillager",
            Card::Potion => "Potion",
            Card::Scabbs => "Scabbs",
            Card::Shark => "Shark",
        })
    }
}

impl Card {
    pub fn cost(&self) -> i32 {
        match self {
            Card::Coin => 0,
            Card::Dancer => 2,
            Card::Foxy => 2,
            Card::Pillager => 6,
            Card::Potion => 4,
            Card::Scabbs => 4,
            Card::Shark => 4,
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
}

// Properties that apply to only the specific version of this card, in our hand.
// This could extend to on-board properties later.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct CardInstance {
    pub card: Card,
    pub potion: bool,
    pub tenwu: bool,
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
        }
    }

    pub fn cost(&self) -> i32 {
        if self.potion || self.tenwu {
            1
        } else {
            self.card.cost()
        }
    }
}
