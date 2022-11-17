use std::fmt;

enum Card {
    Coin,
    Dancer,
    Foxy,
    Pillager,
    Potion,
    Scabbs,
    Shark,
}

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
    fn cost(&self) -> i32 {
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

    fn minion(&self) -> bool {
        match self {
            Card::Dancer => true,
            Card::Foxy => true,
            Card::Pillager => true,
            Card::Scabbs => true,
            Card::Shark => true,
            _ => false,
        }
    }

    fn combo(&self) -> bool {
        match self {
            Card::Pillager => true,
            Card::Scabbs => true,
            _ => false,
        }
    }
}

fn main() {
    let c = Card::Dancer;
    println!(
        "{} has cost {}, minion {}, combo {}",
        c,
        c.cost(),
        c.minion(),
        c.combo()
    );
}
