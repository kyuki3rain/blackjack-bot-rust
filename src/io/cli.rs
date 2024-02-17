use tokio::sync::mpsc::{Receiver, Sender};

use crate::utils::{command::Command, message::Message};

mod input;
mod output;

pub async fn run(tx: &Sender<Command>, rx: &mut Receiver<Message>) -> Result<(), String> {
    let input = input::run(tx);
    let output = output::run(rx);

    tokio::join!(input, output).0.map_err(|e| e.to_string())
}
