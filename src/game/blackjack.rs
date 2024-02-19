use tokio::sync::mpsc::{Receiver, Sender};

use crate::utils::message::Message;

use self::command::{GameCommand, GameCommandData};

pub mod card;
pub mod command;
mod deck;
mod game;
mod manager;
mod player;
mod status;

pub async fn run(tx: &Sender<Message>, rx: &mut Receiver<GameCommand>) -> Result<(), String> {
    let mut players = vec![];
    loop {
        let command = rx.recv().await.unwrap();
        let name = command.user_name;

        if let GameCommandData::Participate = &command.data {
            players.push(name.clone());
        } else {
            continue;
        }

        loop {
            let result = manager::manager(tx, rx, players).await.unwrap();

            players = vec![];
            for (name, _) in result {
                players.push(name);
            }

            if players.is_empty() {
                break;
            }
        }
    }
}
