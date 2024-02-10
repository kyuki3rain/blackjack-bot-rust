use crate::card::Card;

#[derive(Debug, Clone)]
pub struct Player {
    name: String,
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

    pub fn bet(&mut self, amount: u32) {
        self.amount += amount;
    }

    pub fn add_card(&mut self, card: Card) {
        self.hands.push(card);
    }

    pub fn get_hands_symbol(&self, hide_first: bool) -> String {
        let mut symbol = String::new();
        for (i, card) in self.hands.iter().enumerate() {
            if i == 0 && hide_first {
                symbol += "XX ";
            } else {
                symbol += &card.get_symbol();
                symbol += " ";
            }
        }
        symbol
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
