use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::Sender;

use crate::db::user_id::UserId;
use crate::utils::command::{Command, CommandData};

pub async fn run(tx: &Sender<Command>) -> Result<(), io::Error> {
    let stdin = io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    while let Some(line) = lines.next_line().await? {
        let input = line.trim().to_string();
        let command = match parse_input(&input) {
            Ok(command) => command,
            Err(err) => {
                continue;
            }
        };

        if let CommandData::Exit = command.data {
            break;
        }

        tx.send(command).await.unwrap();
    }

    Ok(())
}

pub fn parse_input(input: &str) -> Result<Command, String> {
    let mut iter = input.split_whitespace();
    let command = iter.next().ok_or("Command is missing")?;
    let name = iter.next().ok_or("Name is missing")?.to_string();
    let user_id = UserId::Cli(name.clone());

    match command.to_lowercase().as_str() {
        "register" => Ok(Command::new(CommandData::Register(name.clone()), user_id)),
        "rename" => {
            let new_name = iter.next().ok_or("Name is missing")?.to_string();
            Ok(Command::new(CommandData::Rename(new_name), user_id))
        }
        "balance" => Ok(Command::new(CommandData::Balance, user_id)),
        "bonus" => {
            let amount = iter
                .next()
                .ok_or("Amount is missing")?
                .parse()
                .map_err(|_| "Amount should be a number".to_string())?;
            Ok(Command::new(CommandData::GetBonus(amount), user_id))
        }
        "participate" => Ok(Command::new(CommandData::Participate, user_id)),
        "leave" => Ok(Command::new(CommandData::Leave, user_id)),
        "bet" => {
            let amount = iter
                .next()
                .ok_or("Amount is missing")?
                .parse()
                .map_err(|_| "Amount should be a number".to_string())?;
            Ok(Command::new(CommandData::Bet(amount), user_id))
        }
        "hit" => Ok(Command::new(CommandData::Hit, user_id)),
        "stand" => Ok(Command::new(CommandData::Stand, user_id)),
        "exit" => Ok(Command::new(CommandData::Exit, user_id)),
        _ => Err("Unknown command".into()),
    }
}
