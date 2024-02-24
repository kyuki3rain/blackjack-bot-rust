use std::{fmt, str::FromStr};

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

    pub fn new_hidden() -> Card {
        Card {
            suit: Suit::Hidden,
            value: Value::Hidden,
        }
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
            Value::Hidden => 0,
        }
    }

    pub fn is_ace(&self) -> bool {
        matches!(self.value, Value::Ace)
    }

    pub fn ace_diff() -> u32 {
        let card = Card::new(Suit::Spade, Value::Ace);

        card.get_score(true) - card.get_score(false)
    }
}

impl FromStr for Card {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 10の場合もあるので、最初の1文字をスートとして取得する
        let suit = Suit::from_str(&s[0..1])?;

        // 10の場合は2文字目から取得する
        let value = if s.len() == 3 {
            Value::from_str(&s[1..3])?
        } else {
            Value::from_str(&s[1..2])?
        };

        Ok(Card::new(suit, value))
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.suit.to_string(), self.value.to_string())
    }
}

#[derive(Debug, EnumIter, Copy, Clone, PartialEq)]
pub enum Suit {
    Spade,
    Heart,
    Diamond,
    Club,
    Hidden,
}

impl FromStr for Suit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "S" => Ok(Suit::Spade),
            "H" => Ok(Suit::Heart),
            "D" => Ok(Suit::Diamond),
            "C" => Ok(Suit::Club),
            "X" => Ok(Suit::Hidden), // "X"は不明なカードを表す
            _ => Err("Invalid suit".to_string()),
        }
    }
}

impl ToString for Suit {
    fn to_string(&self) -> String {
        match self {
            Suit::Spade => "S",
            Suit::Heart => "H",
            Suit::Diamond => "D",
            Suit::Club => "C",
            Suit::Hidden => "X",
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
    Hidden,
}

impl FromStr for Value {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(Value::Ace),
            "2" => Ok(Value::Two),
            "3" => Ok(Value::Three),
            "4" => Ok(Value::Four),
            "5" => Ok(Value::Five),
            "6" => Ok(Value::Six),
            "7" => Ok(Value::Seven),
            "8" => Ok(Value::Eight),
            "9" => Ok(Value::Nine),
            "10" => Ok(Value::Ten),
            "J" => Ok(Value::Jack),
            "Q" => Ok(Value::Queen),
            "K" => Ok(Value::King),
            "X" => Ok(Value::Hidden), // "X"は不明なカードを表す
            _ => Err("Invalid value".to_string()),
        }
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
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
            Value::Hidden => "X",
        }
        .to_string()
    }
}

// カードのparseテスト
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_card() {
        let card = Card::from_str("SA").unwrap();
        assert_eq!(card.suit, Suit::Spade);
        assert_eq!(card.value, Value::Ace);

        let card = Card::from_str("H10").unwrap();
        assert_eq!(card.suit, Suit::Heart);
        assert_eq!(card.value, Value::Ten);
    }
}
