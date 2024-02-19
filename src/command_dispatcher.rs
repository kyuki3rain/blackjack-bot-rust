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
        let result = command_dispatcher(command, game_tx.clone(), message_tx.clone()).await;
        if let Err(err) = result {
            message_tx
                .send(Message::Error(err.to_string()))
                .await
                .unwrap();
        }
    }
}

async fn command_dispatcher(
    command: Command,
    game_tx: Sender<GameCommand>,
    message_tx: Sender<Message>,
) -> Result<(), sqlx::Error> {
    let user_id = command.user_id;

    match command.data {
        CommandData::Register(name) => {
            let name = database::register(user_id, name).await?;
            message_tx.send(Message::Register(name)).await.unwrap();
        }
        CommandData::Rename(name) => {
            let (old_name, new_name) = database::rename(user_id, name).await?;
            message_tx
                .send(Message::Rename((old_name, new_name)))
                .await
                .unwrap();
        }
        CommandData::Balance => {
            let (name, balance) = database::balance(user_id).await?;
            message_tx
                .send(Message::Balance((name, balance)))
                .await
                .unwrap();
        }
        CommandData::GetBonus(bonus_id) => {
            let (name, amount) = database::bonus(user_id, bonus_id).await?;
            message_tx
                .send(Message::GetBonus((name, amount)))
                .await
                .unwrap();
        }
        CommandData::Participate => {
            let name = database::get_name(user_id).await?;
            game_tx
                .send(GameCommand::new(GameCommandData::Participate, name))
                .await
                .unwrap();
        }
        CommandData::Leave => {
            let name = database::get_name(user_id).await?;
            game_tx
                .send(GameCommand::new(GameCommandData::Leave, name))
                .await
                .unwrap();
        }
        CommandData::Bet(amount) => {
            let name = database::bet(user_id, amount as i32).await?;
            game_tx
                .send(GameCommand::new(GameCommandData::Bet(amount), name))
                .await
                .unwrap();
        }
        CommandData::Hit => {
            let name = database::get_name(user_id).await?;
            game_tx
                .send(GameCommand::new(GameCommandData::Hit, name))
                .await
                .unwrap();
        }
        CommandData::Stand => {
            let name = database::get_name(user_id).await?;
            game_tx
                .send(GameCommand::new(GameCommandData::Stand, name))
                .await
                .unwrap();
        }
        CommandData::Exit => {}
    }

    Ok(())
}
