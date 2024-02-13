use tokio::sync::mpsc::Sender;

use crate::command::{Command, CommandData};

pub const INPUT_ID: &str = "cli";

pub async fn input_listener(tx: Sender<Command>) {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_string();

        let command = match input.parse::<CommandData>() {
            Ok(data) => Command::new(INPUT_ID, data),
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        tx.send(command.clone()).await.unwrap();

        if let CommandData::Exit = command.data {
            break;
        }
    }
}
