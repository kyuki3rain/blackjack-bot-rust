use std::{collections::HashMap, fmt};

use super::{card::Card, player::Player, status::Status};

pub struct State {
    players: HashMap<String, Player>,
    dealer: Player,
    player_order: Vec<String>,
    status: Status,
}

#[derive(Debug, Clone)]
pub enum Effect {
    Init(Vec<String>),
    AddPlayer(String),
    RemovePlayer(String),
    Bet(String, u32),
    Deal(HashMap<String, (Card, Card)>, (Card, Card)),
    DealerBlackjack,
    Start,
    AddCard(String, Card),
    AddDealerCard(Card),
    OpenDealerCard(Card),
    Burst(String),
    DealerBurst,
    NextPlayer,
    NoPlayer,
    Finish,
}

impl State {
    pub fn new() -> State {
        State {
            players: HashMap::new(),
            dealer: Player::new("ディーラー".to_string()),
            player_order: Vec::new(),
            status: Status::Betting,
        }
    }

    pub fn apply_effect(&mut self, effect: Effect) {
        match effect {
            Effect::Init(players) => {
                self.players.clear();
                self.player_order.clear();
                self.dealer.clear();
                self.status = Status::Betting;
                for player in players {
                    self.add_player(player);
                }
            }
            Effect::AddPlayer(name) => self.add_player(name),
            Effect::RemovePlayer(name) => self.remove_player(&name),
            Effect::Bet(name, amount) => self.bet(&name, amount),
            Effect::Deal(player_cards, dealer_cards) => self.deal(player_cards, dealer_cards),
            Effect::DealerBlackjack => {}
            Effect::Start => self.start(),
            Effect::AddCard(name, card) => self.add_card(&name, card),
            Effect::AddDealerCard(card) => self.add_dealer_card(card),
            Effect::OpenDealerCard(card) => self.open_dealer_card(card),
            Effect::Burst(_) => {}
            Effect::DealerBurst => {}
            Effect::NextPlayer => self.next_player(),
            Effect::NoPlayer => self.finish(),
            Effect::Finish => self.finish(),
        }
    }

    fn add_player(&mut self, name: String) {
        self.players.insert(name.clone(), Player::new(name.clone()));
        self.player_order.push(name);
    }

    fn remove_player(&mut self, name: &str) {
        self.players.remove(name);
        self.player_order.retain(|x| x != name);
    }

    fn bet(&mut self, name: &str, amount: u32) {
        self.players.get_mut(name).unwrap().bet(amount);
    }

    fn deal(&mut self, player_cards: HashMap<String, (Card, Card)>, dealer_cards: (Card, Card)) {
        for (name, card) in player_cards {
            self.add_card(&name, card.0);
            self.add_card(&name, card.1);
        }
        self.add_dealer_card(dealer_cards.0);
        self.add_dealer_card(dealer_cards.1);
    }

    fn start(&mut self) {
        self.status = Status::Dealing;
    }

    fn add_card(&mut self, name: &str, card: Card) {
        self.players.get_mut(name).unwrap().add_card(card);
    }

    fn add_dealer_card(&mut self, card: Card) {
        self.dealer.add_card(card);
    }

    fn open_dealer_card(&mut self, card: Card) {
        self.dealer.open_card(card);
    }

    fn next_player(&mut self) {
        match self.status {
            Status::Playing(i) => {
                if i + 1 < self.player_order.len() {
                    self.status = Status::Playing(i + 1);
                } else {
                    self.status = Status::DealerTurn;
                }
            }
            Status::Dealing => {
                self.status = Status::Playing(0);
            }
            _ => {}
        }
    }

    fn finish(&mut self) {
        self.status = Status::End;
    }

    pub fn get_current_player(&self) -> Option<&Player> {
        if let Status::Playing(i) = self.status {
            self.players.get(&self.player_order[i])
        } else {
            None
        }
    }

    pub fn get_player_order(&self) -> Vec<String> {
        self.player_order.clone()
    }

    pub fn get_player_count(&self) -> usize {
        self.players.len()
    }

    pub fn get_player(&self, name: &str) -> Option<&Player> {
        self.players.get(name)
    }

    pub fn get_dealer(&self) -> &Player {
        &self.dealer
    }

    pub fn get_dealer_hands(&self, hide: bool) -> Vec<Card> {
        self.dealer.get_hands(hide)
    }

    pub fn get_dealer_score(&self) -> u32 {
        self.dealer.get_score()
    }

    pub fn is_betting(&self) -> bool {
        self.status == Status::Betting
    }

    pub fn is_dealer_turn(&self) -> bool {
        self.status == Status::DealerTurn
    }

    pub fn has_player(&self, name: &str) -> bool {
        self.players.contains_key(name)
    }

    pub fn is_started(&self) -> bool {
        self.status != Status::Betting
    }

    pub fn is_finished(&self) -> bool {
        self.status == Status::End
    }

    pub fn get_result(&self) -> HashMap<String, (u32, i32)> {
        let mut result = HashMap::new();
        let dealer_score = self.dealer.get_score();
        for name in &self.player_order {
            let player_score = self.players[name].get_score();
            let player_amount = self.players[name].get_amount();
            let score = if player_score > 21 {
                0
            } else if dealer_score > 21 || player_score > dealer_score {
                2
            } else if player_score < dealer_score {
                0
            } else {
                1
            } * player_amount;
            result.insert(name.clone(), (score, score as i32 - player_amount as i32));
        }
        result
    }

    pub fn get_amounts(&self) -> HashMap<String, u32> {
        let mut amounts = HashMap::new();
        for name in &self.player_order {
            amounts.insert(name.clone(), self.players[name].get_amount());
        }
        amounts
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.dealer).and_then(|_| {
            for name in &self.player_order {
                writeln!(f, "{}", self.players[name])?;
            }
            Ok(())
        })
    }
}
