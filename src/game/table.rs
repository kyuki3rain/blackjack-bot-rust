use super::{
    card::Card,
    deck::Deck,
    state::{Effect, State},
};

#[derive(Debug, Clone)]
pub enum Command {
    Ping(String),
    Participate(String),
    Leave(String),
    Bet(String, u32),
    Hit(String),
    Stand(String),
}

impl Command {
    pub fn success_message(&self) -> String {
        match self {
            Command::Ping(name) => format!("pong, {}", name),
            Command::Participate(name) => format!("{name}さんが参加しました。"),
            Command::Leave(name) => format!("{name}さんが退出しました。"),
            Command::Bet(name, amount) => format!("{name}さんが{amount}コイン賭けました。"),
            Command::Hit(name) => format!("{name}さんがヒットしました。"),
            Command::Stand(name) => format!("{name}さんがスタンドしました。"),
        }
    }
}

pub struct Table {
    state: State,
    deck: Deck,
}

impl Table {
    pub fn new() -> Table {
        let mut deck = Deck::new();
        deck.shuffle();

        Table {
            state: State::new(),
            deck,
        }
    }

    pub fn init_players(&mut self, players: Vec<String>) {
        let effect = Effect::Init(players);
        self.state.apply_effect(effect);
    }

    pub fn apply_command(&mut self, command: Command) -> Result<Vec<Effect>, String> {
        match command {
            Command::Ping(_) => Ok(vec![]),
            Command::Participate(name) => self.participate(name),
            Command::Leave(name) => self.leave(&name),
            Command::Bet(name, amount) => self.bet(&name, amount),
            Command::Hit(name) => self.hit(&name),
            Command::Stand(name) => self.stand(&name),
        }
    }

    fn participate(&mut self, name: String) -> Result<Vec<Effect>, String> {
        if !self.state.is_betting() {
            return Err("Game has already started".to_string());
        }

        if self.state.has_player(&name) {
            return Err("Player already exists".to_string());
        }

        let effect = Effect::AddPlayer(name.clone());
        self.state.apply_effect(effect.clone());
        Ok(vec![effect])
    }

    fn leave(&mut self, name: &str) -> Result<Vec<Effect>, String> {
        if !self.state.is_betting() {
            return Err("Game has already started".to_string());
        }

        if !self.state.has_player(name) {
            return Err("Player does not exist".to_string());
        }

        let effect = Effect::RemovePlayer(name.to_string());
        self.state.apply_effect(effect.clone());
        Ok(vec![effect])
    }

    fn bet(&mut self, name: &str, amount: u32) -> Result<Vec<Effect>, String> {
        if !self.state.is_betting() {
            return Err("Game has already started".to_string());
        }

        if !self.state.has_player(name) {
            return Err("Player does not exist".to_string());
        }

        let effect = Effect::Bet(name.to_string(), amount);
        self.state.apply_effect(effect.clone());
        Ok(vec![effect])
    }

    pub fn start(&mut self) -> Result<Vec<Effect>, String> {
        if !self.state.is_betting() {
            return Err("Game has already started".to_string());
        }

        if self.state.get_player_count() == 0 {
            return Err("No player".to_string());
        }

        let mut effects = vec![];

        let effect = Effect::Start;
        self.state.apply_effect(effect.clone());
        effects.push(effect);

        let mut player_cards = std::collections::HashMap::new();
        for name in self.state.get_player_order() {
            let card1 = self.deck.draw().ok_or("Deck is empty".to_string())?;
            let card2 = self.deck.draw().ok_or("Deck is empty".to_string())?;
            player_cards.insert(name.clone(), (card1, card2));
        }
        let dealer_card1 = self.deck.draw().ok_or("Deck is empty".to_string())?;
        let dealer_card2 = self.deck.draw().ok_or("Deck is empty".to_string())?;
        let dummy = Card::new_hidden();

        let effect = Effect::Deal(player_cards.clone(), (dealer_card1, dealer_card2));
        self.state.apply_effect(effect);
        let dummy_effect = Effect::Deal(player_cards, (dealer_card1, dummy));
        effects.push(dummy_effect);

        if self.state.get_dealer_score() == 21 {
            let effect = Effect::DealerBlackjack;
            self.state.apply_effect(effect.clone());
            effects.push(effect);
            let effect = Effect::Finish;
            self.state.apply_effect(effect.clone());
            effects.push(effect);
            let effect = Effect::Init(self.state.get_player_order());
            self.state.apply_effect(effect.clone());
            effects.push(effect);
        } else {
            let effect = Effect::NextPlayer;
            self.state.apply_effect(effect.clone());
            effects.push(effect);
        }

        Ok(effects)
    }

    fn hit(&mut self, name: &str) -> Result<Vec<Effect>, String> {
        match self.state.get_current_player() {
            Some(player) => {
                if player.name != name {
                    return Err("It's not your turn".to_string());
                }
            }
            None => return Err("Game has not started yet".to_string()),
        }

        let mut effects = vec![];

        let card = self.deck.draw().ok_or("Deck is empty".to_string())?;
        let effect = Effect::AddCard(name.to_string(), card);
        self.state.apply_effect(effect.clone());
        effects.push(effect);

        if self.state.get_current_player().unwrap().get_score() > 21 {
            effects.push(Effect::Burst(name.to_string()));
            effects.append(&mut self.stand(name)?);
        }

        Ok(effects)
    }

    fn stand(&mut self, name: &str) -> Result<Vec<Effect>, String> {
        match self.state.get_current_player() {
            Some(player) => {
                if player.name != name {
                    return Err("It's not your turn".to_string());
                }
            }
            None => return Err("Game has not started yet".to_string()),
        }

        let effect = Effect::NextPlayer;
        self.state.apply_effect(effect.clone());
        Ok(vec![effect])
    }

    pub fn dealer_action(&mut self) -> Result<Vec<Effect>, String> {
        if !self.state.is_dealer_turn() {
            return Err("It's not dealer's turn".to_string());
        }

        let mut effects = vec![];

        let hidden_card = *self.state.get_dealer_hands(false).get(1).unwrap();
        let effect = Effect::OpenDealerCard(hidden_card);
        self.state.apply_effect(effect.clone());
        effects.push(effect);

        while self.state.get_dealer_score() < 17 {
            let card = self.deck.draw().ok_or("Deck is empty".to_string())?;
            let effect = Effect::AddDealerCard(card);
            self.state.apply_effect(effect.clone());
            effects.push(effect);
        }

        if self.state.get_dealer_score() > 21 {
            let effect = Effect::DealerBurst;
            self.state.apply_effect(effect.clone());
            effects.push(effect);
        }

        let effect = Effect::Finish;
        self.state.apply_effect(effect.clone());
        effects.push(effect);

        Ok(effects)
    }

    pub fn is_dealer_turn(&self) -> bool {
        self.state.is_dealer_turn()
    }

    pub fn is_started(&self) -> bool {
        self.state.is_started()
    }

    pub fn is_finished(&self) -> bool {
        self.state.is_finished()
    }

    pub fn get_player_count(&self) -> usize {
        self.state.get_player_count()
    }

    pub fn get_players(&self) -> Vec<String> {
        self.state.get_player_order()
    }

    pub fn get_player_order(&self) -> Vec<String> {
        self.state.get_player_order()
    }
}
