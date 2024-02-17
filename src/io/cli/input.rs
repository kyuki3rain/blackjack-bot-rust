use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::Sender;

use crate::db::user_id::UserId;
use crate::utils::command::{Command, CommandData};

pub async fn run(tx: &Sender<Command>) -> Result<(), io::Error> {
    let stdin = io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    while let Some(line) = lines.next_line().await? {
        let input = line.trim().to_string();
        let command = parse_input(&input).unwrap();

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

    match command.to_lowercase().as_str() {
        "register" => Ok(Command::new(
            CommandData::Register(name.clone()),
            UserId::Cli(name),
        )),
        "rename" => {
            let new_name = iter.next().ok_or("Name is missing")?.to_string();
            Ok(Command::new(
                CommandData::Rename(name),
                UserId::Cli(new_name),
            ))
        }
        "balance" => Ok(Command::new(CommandData::Balance, UserId::Cli(name))),
        "bonus" => {
            let amount = iter
                .next()
                .ok_or("Amount is missing")?
                .parse()
                .map_err(|_| "Amount should be a number".to_string())?;
            Ok(Command::new(
                CommandData::GetBonus(amount),
                UserId::Cli(name),
            ))
        }
        "participate" => Ok(Command::new(CommandData::Participate, UserId::Cli(name))),
        "leave" => Ok(Command::new(CommandData::Leave, UserId::Cli(name))),
        "bet" => {
            let amount = iter
                .next()
                .ok_or("Amount is missing")?
                .parse()
                .map_err(|_| "Amount should be a number".to_string())?;
            Ok(Command::new(CommandData::Bet(amount), UserId::Cli(name)))
        }
        "hit" => Ok(Command::new(CommandData::Hit, UserId::Cli(name))),
        "stand" => Ok(Command::new(CommandData::Stand, UserId::Cli(name))),
        "exit" => Ok(Command::new(CommandData::Exit, UserId::Cli(name))),
        _ => Err("Unknown command".into()),
    }
}
