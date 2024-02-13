use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Command {
    pub from: &'static str,
    pub data: CommandData,
}

impl Command {
    pub fn new(from: &'static str, data: CommandData) -> Command {
        Command { from, data }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandData {
    Participate(String),
    Leave(String),
    Bet(String, u32),
    Hit(String),
    Stand(String),
    Start,
    Exit,
}

impl FromStr for CommandData {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace();

        // unwrap or Err
        let content_str = if let Some(content_str) = iter.next() {
            content_str
        } else {
            return Err("コマンドを入力してください。".to_string());
        };

        let name = iter.next();
        let name_err = "名前を入力してください。".to_string();

        match content_str {
            "/participate" => Ok(CommandData::Participate(name.ok_or(name_err)?.to_string())),
            "/leave" => Ok(CommandData::Leave(name.ok_or(name_err)?.to_string())),
            "/bet" => {
                let amount: u32 = if let Some(amount_str) = iter.next() {
                    if let Ok(amount) = amount_str.parse() {
                        amount
                    } else {
                        return Err("金額を入力してください。".to_string());
                    }
                } else {
                    return Err("金額を入力してください。".to_string());
                };
                Ok(CommandData::Bet(name.ok_or(name_err)?.to_string(), amount))
            }
            "/hit" => Ok(CommandData::Hit(name.ok_or(name_err)?.to_string())),
            "/stand" => Ok(CommandData::Stand(name.ok_or(name_err)?.to_string())),
            "/exit" => Ok(CommandData::Exit),
            _ => Err("不正なコマンドです。".to_string()),
        }
    }
}
