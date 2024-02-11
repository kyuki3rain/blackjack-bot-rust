use crate::{
    blackjack,
    command::{Command, CommandResult},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub async fn play(txs: Vec<Sender<String>>, mut rx: Receiver<String>) {
    let mut game = blackjack::Blackjack::new();

    loop {
        let command_str = rx.recv().await.unwrap();
        let command: Command = command_str.parse().unwrap();
        let tx = &txs[command.from];

        let result = match command.content {
            Ok(content) => {
                let content = game.execute(content);
                CommandResult {
                    content,
                    to: command.from,
                }
            }
            Err(err) => CommandResult {
                content: Err(err),
                to: command.from,
            },
        };

        tx.send(result.to_string()).await.unwrap();
    }
}
