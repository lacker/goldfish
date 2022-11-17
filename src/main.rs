enum Card {
    Coin,
    Dancer,
    Foxy,
    Pillager,
    Potion,
    Scabbs,
    Shark,
}

fn cost(card: &Card) -> i32 {
    match card {
        Card::Coin => 0,
        Card::Dancer => 2,
        Card::Foxy => 2,
        Card::Pillager => 6,
        Card::Potion => 4,
        Card::Scabbs => 4,
        Card::Shark => 4,
    }
}

fn main() {
    println!("cost of dancer: {}", cost(&Card::Dancer));
}
