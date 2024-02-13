use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    command::{Command, CommandData},
    message::{Message, MessageData},
};

pub const TIMER_ID: &str = "timer";

pub async fn timer(tx: Sender<Command>, mut rx: Receiver<Message>) {
    loop {
        let message = rx.recv().await.unwrap();
        if let MessageData::Init(time) = message.data {
            tokio::time::sleep(tokio::time::Duration::from_secs(time)).await;
            tx.send(Command::new(TIMER_ID, CommandData::Start))
                .await
                .unwrap();
        }
        if let MessageData::Exit = message.data {
            break;
        }
    }
}
