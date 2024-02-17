use crate::game::blackjack::card::{Card, Suit, Value};
use rand::{seq::SliceRandom, thread_rng};
use strum::IntoEnumIterator;

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Deck {
        let mut cards = Vec::new();
        for suit in Suit::iter() {
            if Suit::Hidden == suit {
                continue;
            }

            for value in Value::iter() {
                if Value::Hidden == value {
                    continue;
                }

                cards.push(Card::new(suit, value));
            }
        }
        Deck { cards }
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}
