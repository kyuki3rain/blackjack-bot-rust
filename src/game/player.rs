use super::card::Card;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    hands: Vec<Card>,
    amount: u32,
}

impl Player {
    pub fn new(name: String) -> Player {
        Player {
            name,
            hands: Vec::new(),
            amount: 0,
        }
    }

    pub fn clear(&mut self) {
        self.hands.clear();
        self.amount = 0;
    }

    pub fn bet(&mut self, amount: u32) {
        self.amount += amount;
    }

    pub fn get_hands(&self, hide: bool) -> Vec<Card> {
        if hide {
            let mut hands = self.hands.clone();
            hands[1] = Card::new_hidden();
            hands
        } else {
            self.hands.clone()
        }
    }

    pub fn add_card(&mut self, card: Card) {
        self.hands.push(card);
    }

    pub fn open_card(&mut self, card: Card) {
        self.hands[1] = card;
    }

    pub fn get_score(&self) -> u32 {
        let mut score = 0;
        let mut ace_count = 0;
        for card in &self.hands {
            let card_score = card.get_score(true);
            if card.is_ace() {
                ace_count += 1;
            }
            score += card_score;
        }

        let ace_diff = Card::ace_diff();
        while score > 21 && ace_count > 0 {
            score -= ace_diff;
            ace_count -= 1;
        }

        score
    }

    pub fn get_amount(&self) -> u32 {
        self.amount
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:", self.name)?;
        for card in &self.hands {
            write!(f, " {}", card)?;
        }
        write!(f, " ({})", self.get_score())
    }
}
