use tokio::sync::mpsc::Receiver;

use crate::message::{Message, MessageData};

pub const DISPLAY_ID: &str = "cli";

pub async fn display(mut rx: Receiver<Message>) {
    loop {
        let message = rx.recv().await.unwrap();
        println!("{}", message.data.to_string());
        if let MessageData::Exit = message.data {
            break;
        }
    }
}
