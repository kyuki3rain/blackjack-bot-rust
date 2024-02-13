use std::collections::HashMap;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::message::{Message, MessageData};

pub const BROADCAST_ID: &str = "broadcast";

pub async fn dispatcher(txs: HashMap<&str, Sender<Message>>, mut rx: Receiver<Message>) {
    loop {
        let message = rx.recv().await.unwrap();

        if message.to == BROADCAST_ID {
            for tx in txs.values() {
                tx.send(message.clone()).await.unwrap();
            }
        } else {
            match txs.get(&message.to) {
                Some(tx) => {
                    tx.send(message.clone()).await.unwrap();
                }
                None => {
                    println!("No receiver found: {}", message.to);
                    continue;
                }
            };
        }

        if let MessageData::Exit = message.data {
            break;
        }
    }
}
