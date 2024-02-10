use strum::EnumIter;

#[derive(Debug, Clone, Copy)]
pub struct Card {
    suit: Suit,
    value: Value,
}

impl Card {
    pub fn new(suit: Suit, value: Value) -> Card {
        Card { suit, value }
    }

    pub fn get_score(&self, ace_high: bool) -> u32 {
        match self.value {
            Value::Ace => {
                if ace_high {
                    11
                } else {
                    1
                }
            }
            Value::Two => 2,
            Value::Three => 3,
            Value::Four => 4,
            Value::Five => 5,
            Value::Six => 6,
            Value::Seven => 7,
            Value::Eight => 8,
            Value::Nine => 9,
            Value::Ten => 10,
            Value::Jack => 10,
            Value::Queen => 10,
            Value::King => 10,
        }
    }

    pub fn is_ace(&self) -> bool {
        matches!(self.value, Value::Ace)
    }

    pub fn ace_diff() -> u32 {
        let card = Card::new(Suit::Spade, Value::Ace);

        card.get_score(true) - card.get_score(false)
    }

    pub fn get_symbol(&self) -> String {
        format!("{}{}", self.value.get_symbol(), self.suit.get_symbol())
    }
}

#[derive(Debug, EnumIter, Copy, Clone, PartialEq)]
pub enum Suit {
    Spade,
    Heart,
    Diamond,
    Club,
}

impl Suit {
    fn get_symbol(&self) -> String {
        match self {
            Suit::Spade => "♠",
            Suit::Heart => "♥",
            Suit::Diamond => "♦",
            Suit::Club => "♣",
        }
        .to_string()
    }
}

#[derive(Debug, EnumIter, Copy, Clone, PartialEq)]
pub enum Value {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Value {
    fn get_symbol(&self) -> String {
        match self {
            Value::Ace => "A",
            Value::Two => "2",
            Value::Three => "3",
            Value::Four => "4",
            Value::Five => "5",
            Value::Six => "6",
            Value::Seven => "7",
            Value::Eight => "8",
            Value::Nine => "9",
            Value::Ten => "10",
            Value::Jack => "J",
            Value::Queen => "Q",
            Value::King => "K",
        }
        .to_string()
    }
}
