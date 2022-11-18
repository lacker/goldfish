use std::fmt;

// Properties that apply to a card wherever it is
#[derive(Copy, Clone)]
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

// Properties that apply to only the specific version of this card, in our hand.
// This could extend to on-board properties later.
#[derive(Copy, Clone)]
struct CardInstance {
    card: Card,
    potion: bool,
    tenwu: bool,
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

#[derive(Clone)]
struct Game {
    board: Vec<Card>,        // our side of the board
    hand: Vec<CardInstance>, // our hand
    life: i32,               // the opponent's life
    mana: i32,               // our mana
    storm: i32,              // number of things played this turn
    foxy: i32,               // number of stacks of the foxy effect
    scabbs: i32,             // number of stacks of the scabbs effect
    next_scabbs: i32,        // number of stacks of the scabbs effect after this one
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "board: {}",
            self.board
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
        .unwrap();
        writeln!(
            f,
            "hand: {}",
            self.hand
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
        .unwrap();
        writeln!(f, "life: {}", self.life).unwrap();
        writeln!(f, "mana: {}", self.mana).unwrap();
        if self.storm > 0 {
            writeln!(f, "storm: {}", self.storm).unwrap();
        }
        if self.foxy > 0 {
            writeln!(f, "foxy: {}", self.foxy).unwrap();
        }
        if self.scabbs > 0 {
            writeln!(f, "scabbs: {}", self.scabbs).unwrap();
        }
        if self.next_scabbs > 0 {
            writeln!(f, "next_scabbs: {}", self.next_scabbs).unwrap();
        }
        Ok(())
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Vec::new(),
            hand: Vec::new(),
            life: 30,
            mana: 0,
            storm: 0,
            foxy: 0,
            scabbs: 0,
            next_scabbs: 0,
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
