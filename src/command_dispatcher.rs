use tokio::sync::mpsc::{Receiver, Sender};

use crate::db::database;
use crate::game::blackjack::command::{GameCommand, GameCommandData};
use crate::utils::command::{Command, CommandData};
use crate::utils::message::Message;

pub async fn run(
    mut rx: Receiver<Command>,
    game_tx: &Sender<GameCommand>,
    message_tx: &Sender<Message>,
) -> Result<(), String> {
    loop {
        let command = rx.recv().await.unwrap();
        eprintln!("command: {:?}", command);
        let user_id = command.user_id;

        match command.data {
            CommandData::Register(name) => {
                println!("Registering user {}", name);
                let name = database::register(user_id, name).await.unwrap();
                message_tx.send(Message::Register(name)).await.unwrap();
            }
            CommandData::Rename(name) => {
                println!("Renaming user to {}", name);
                let (old_name, new_name) = database::rename(user_id, name).await.unwrap();
                message_tx
                    .send(Message::Rename((old_name, new_name)))
                    .await
                    .unwrap();
            }
            CommandData::Balance => {
                println!("Checking balance");
                let (name, balance) = database::balance(user_id).await.unwrap();
                message_tx
                    .send(Message::Balance((name, balance)))
                    .await
                    .unwrap();
            }
            CommandData::GetBonus(bonus_id) => {
                println!("Getting bonus of {}", bonus_id);
                let (name, amount) = database::bonus(user_id, bonus_id).await.unwrap();
                message_tx
                    .send(Message::GetBonus((name, amount)))
                    .await
                    .unwrap();
            }
            CommandData::Participate => {
                let name = database::get_name(user_id).await.unwrap();
                println!("Participating in the game");
                game_tx
                    .send(GameCommand::new(GameCommandData::Participate, name))
                    .await
                    .unwrap();
            }
            CommandData::Leave => {
                let name = database::get_name(user_id).await.unwrap();
                println!("Leaving the game");
                game_tx
                    .send(GameCommand::new(GameCommandData::Leave, name))
                    .await
                    .unwrap();
            }
            CommandData::Bet(amount) => {
                let name = database::bet(user_id, amount as i32).await.unwrap();
                println!("Betting {}", amount);
                game_tx
                    .send(GameCommand::new(GameCommandData::Bet(amount), name))
                    .await
                    .unwrap();
            }
            CommandData::Hit => {
                let name = database::get_name(user_id).await.unwrap();
                println!("Hitting");
                game_tx
                    .send(GameCommand::new(GameCommandData::Hit, name))
                    .await
                    .unwrap();
            }
            CommandData::Stand => {
                let name = database::get_name(user_id).await.unwrap();
                println!("Standing");
                game_tx
                    .send(GameCommand::new(GameCommandData::Stand, name))
                    .await
                    .unwrap();
            }
            CommandData::Exit => break,
        }
    }

    Ok(())
}
