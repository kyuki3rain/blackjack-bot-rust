use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    db::{database, user_id::UserId},
    utils::message::Message,
};

pub async fn run(mut rx: Receiver<Message>, txs: Vec<Sender<Message>>) -> Result<(), String> {
    loop {
        let message = rx.recv().await.unwrap();
        eprintln!("message: {:?}", message);

        if let Message::ShowResult(result) = &message {
            for (name, (amount, _balance)) in result {
                database::refund(UserId::Name(name.clone()), *amount as i32)
                    .await
                    .unwrap();
            }
        }

        for tx in txs.iter() {
            tx.send(message.clone()).await.unwrap();
        }
    }
}
