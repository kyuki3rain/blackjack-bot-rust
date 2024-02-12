use std::collections::HashMap;

use crate::{
    card::Card,
    command::{CommandResultType, CommandType},
    deck::Deck,
    player::Player,
    status::Status,
};

pub struct Blackjack {
    deck: Deck,
    dealer: Player,
    player_map: HashMap<String, Player>,
    player_order: Vec<String>,
    status: Status,
}

impl Blackjack {
    pub fn new() -> Blackjack {
        let mut deck = Deck::new();
        deck.shuffle();

        let player_map = HashMap::new();

        Blackjack {
            deck,
            dealer: Player::new("Dealer".to_string()),
            player_map,
            player_order: vec![],
            status: Status::End,
        }
    }

    pub fn execute(&mut self, command: CommandType) -> Result<CommandResultType, String> {
        match command {
            CommandType::Participate(name) => {
                Ok(CommandResultType::Participate(self.add_player(name)?))
            }
            CommandType::Leave(name) => Ok(CommandResultType::Leave(self.remove_player(name)?)),
            CommandType::Start => Ok(CommandResultType::Start(self.start()?)),
            CommandType::Reset => Ok(CommandResultType::Reset(self.reset()?)),
            CommandType::Finish => Ok(CommandResultType::Finish(self.finish()?)),
            CommandType::Bet(name, amount) => Ok(CommandResultType::Bet(self.bet(name, amount)?)),
            CommandType::Deal => Ok(CommandResultType::Deal(self.deal()?)),
            CommandType::Hit(name) => Ok(CommandResultType::Hit(self.hit(name)?)),
            CommandType::Stand(name) => Ok(CommandResultType::Stand(self.stand(name)?)),
            CommandType::DealerHit => Ok(CommandResultType::DealerHit(self.dealer_hit()?)),
            CommandType::GetAmounts => Ok(CommandResultType::GetAmounts(self.get_amounts())),
            CommandType::GetStatus => Ok(CommandResultType::GetStatus(self.get_status())),
            CommandType::GetPlayerName(index) => Ok(CommandResultType::GetPlayerName(
                self.get_player_name(index),
            )),
            CommandType::GetBoard(hide_first) => {
                Ok(CommandResultType::GetBoard(self.get_board(hide_first)))
            }
            CommandType::GetPlayerHand(name) => Ok(CommandResultType::GetPlayerHand(
                self.get_player_hand(name)?,
            )),
            CommandType::GetDealerHand(hide_first) => Ok(CommandResultType::GetDealerHand(
                self.get_dealer_hand(hide_first),
            )),
            CommandType::GetDealerScore => {
                Ok(CommandResultType::GetDealerScore(self.get_dealer_score()))
            }
            CommandType::GetResult => Ok(CommandResultType::GetResult(self.get_result()?)),
        }
    }

    pub fn reset(&mut self) -> Result<(), String> {
        if self.status != Status::End {
            return Err("Game already started".to_string());
        }

        self.deck = Deck::new();
        self.deck.shuffle();

        self.dealer = Player::new("Dealer".to_string());
        self.status = Status::Betting;

        for player in self.player_map.values_mut() {
            player.reset();
        }

        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), String> {
        if self.status != Status::Betting && self.status != Status::Dealing {
            return Err("Game already started".to_string());
        }
        self.status = Status::End;

        Ok(())
    }

    pub fn add_player(&mut self, name: String) -> Result<String, String> {
        if self.status != Status::Betting && self.status != Status::End {
            return Err("Game already started".to_string());
        }

        self.player_map
            .insert(name.to_owned(), Player::new(name.to_owned()));
        self.player_order.push(name.to_owned());

        Ok(name)
    }

    pub fn remove_player(&mut self, name: String) -> Result<String, String> {
        if self.status != Status::Betting && self.status != Status::End {
            return Err("Game already started".to_string());
        }

        self.player_map.remove(&name);
        self.player_order.retain(|x| x != &name);

        Ok(name)
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.status != Status::Betting {
            return Err("Game already started".to_string());
        }

        self.status = Status::Dealing;

        Ok(())
    }

    pub fn bet(&mut self, name: String, amount: u32) -> Result<(String, u32), String> {
        if self.status != Status::Betting {
            return Err("Game already started".to_string());
        }

        if let Some(player) = self.player_map.get_mut(&name) {
            player.bet(amount);
        }

        Ok((name, amount))
    }

    pub fn deal(&mut self) -> Result<bool, String> {
        if self.status != Status::Dealing {
            return Err("Game already started".to_string());
        }

        for _ in 0..2 {
            for player in self.player_map.values_mut() {
                if let Some(card) = self.deck.draw() {
                    player.add_card(card);
                }
            }
            if let Some(card) = self.deck.draw() {
                self.dealer.add_card(card);
            }
        }

        if self.dealer.get_score() == 21 {
            self.status = Status::End;
            return Ok(true);
        }

        self.status = Status::Playing(0);

        Ok(false)
    }

    pub fn hit(&mut self, name: String) -> Result<(String, Card), String> {
        let player = self.player_map.get_mut(&name).expect("No User");

        if let Status::Playing(i) = self.status {
            if self.player_order[i] != name {
                return Err("Not your turn".to_string());
            }
        } else {
            return Err("Not playing".to_string());
        }

        let card = self.deck.draw().expect("Deck is Over");

        player.add_card(card);

        Ok((name, card))
    }

    pub fn stand(&mut self, name: String) -> Result<(String, u32), String> {
        let player = self.player_map.get_mut(&name).expect("No User");

        let playing_index = if let Status::Playing(i) = self.status {
            if self.player_order[i] != name {
                return Err("Not your turn".to_string());
            }

            i
        } else {
            return Err("Not playing".to_string());
        };

        if playing_index + 1 < self.player_order.len() {
            self.status = Status::Playing(playing_index + 1);
        } else {
            self.status = Status::DealerTurn;
        }

        Ok((name, player.get_score()))
    }

    pub fn dealer_hit(&mut self) -> Result<Option<Card>, String> {
        if self.status != Status::DealerTurn {
            return Err("Not dealer turn".to_string());
        }

        if self.dealer.get_score() >= 17 {
            self.status = Status::End;
            return Ok(None);
        }

        let card = self.deck.draw().expect("Deck is Over");
        self.dealer.add_card(card);

        if self.dealer.get_score() >= 17 {
            self.status = Status::End;
        }

        Ok(Some(card))
    }

    pub fn get_amounts(&self) -> Vec<(String, u32)> {
        self.player_map
            .iter()
            .map(|(name, player)| (name.to_owned(), player.get_amount()))
            .collect()
    }

    pub fn get_status(&self) -> Status {
        self.status
    }

    pub fn get_player_name(&self, index: usize) -> String {
        self.player_order[index].to_owned()
    }

    pub fn get_board(&self, hide_first: bool) -> String {
        let mut board = String::new();

        for (name, player) in &self.player_map {
            board += &format!("{}: {}\n", name, player.get_hands_symbol(false));
        }

        board += &format!("Dealer: {}\n", self.dealer.get_hands_symbol(hide_first));

        board
    }

    pub fn get_player_hand(&self, name: String) -> Result<(String, u32), String> {
        if let Some(player) = self.player_map.get(&name) {
            Ok((player.get_hands_symbol(false), player.get_score()))
        } else {
            Err("No User".to_string())
        }
    }

    pub fn get_dealer_hand(&self, hide_first: bool) -> String {
        self.dealer.get_hands_symbol(hide_first)
    }

    pub fn get_dealer_score(&self) -> u32 {
        self.dealer.get_score()
    }

    pub fn get_result(&self) -> Result<HashMap<String, (u32, i32)>, String> {
        if self.status != Status::End {
            return Err("Game not ended".to_string());
        }

        let dealer_score = self.dealer.get_score();

        let mut result = HashMap::new();

        for (name, player) in &self.player_map {
            let player_score = player.get_score();
            let player_amount = player.get_amount();

            let player_result = if player_score > 21 {
                0
            } else if dealer_score > 21 || player_score > dealer_score {
                player_amount * 2
            } else if player_score == dealer_score {
                player_amount
            } else {
                0
            };
            result.insert(
                name.to_owned(),
                (player_result, player_result as i32 - player_amount as i32),
            );
        }

        Ok(result)
    }
}
