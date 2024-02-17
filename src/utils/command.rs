use crate::db::user_id::UserId;

#[derive(Debug)]
pub struct Command {
    pub data: CommandData,
    pub user_id: UserId,
}

impl Command {
    pub fn new(data: CommandData, user_id: UserId) -> Command {
        Command { data, user_id }
    }
}

#[derive(Debug)]
pub enum CommandData {
    Register(String),
    Rename(String),
    Balance,
    GetBonus(i32),
    Participate,
    Leave,
    Bet(u32),
    Hit,
    Stand,
    Exit,
}
