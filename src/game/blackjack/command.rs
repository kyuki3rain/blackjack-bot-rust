#[derive(Debug)]
pub struct GameCommand {
    pub user_name: String,
    pub data: GameCommandData,
}

impl GameCommand {
    pub fn new(data: GameCommandData, user_name: String) -> GameCommand {
        GameCommand { user_name, data }
    }
}

#[derive(Debug)]
pub enum GameCommandData {
    Participate,
    Leave,
    Bet(u32),
    Hit,
    Stand,
}
