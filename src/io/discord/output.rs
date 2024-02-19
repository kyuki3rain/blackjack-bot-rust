use std::{env, sync::Arc};

use dotenvy::dotenv;
use serenity::{all::ChannelId, http::Http};
use tokio::sync::mpsc::Receiver;

use crate::utils::message::Message;

pub async fn run(http: Arc<Http>, rx: &mut Receiver<Message>) -> Result<(), String> {
    dotenv().ok();

    loop {
        let message = rx.recv().await.unwrap();
        let channel_id = ChannelId::new(env::var("CHANNEL_ID").unwrap().parse().unwrap());
        channel_id.say(&http, message.to_string()).await.unwrap();
    }
}
