use dotenvy::dotenv;
use serenity::all::GatewayIntents;
use serenity::Client;
use std::env;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

use crate::utils::command::Command;
use crate::utils::message::Message;

use self::input::Handler;

pub mod input;
pub mod output;

pub async fn run(tx: Sender<Command>, rx: &mut Receiver<Message>) -> Result<(), String> {
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler {
            tx: Mutex::new(tx.clone()),
        })
        .await
        .expect("Error creating client");

    let output = output::run(client.http.clone(), rx);
    let input = input::run(&mut client);

    tokio::join!(output, input).0.map_err(|e| e.to_string())
}
