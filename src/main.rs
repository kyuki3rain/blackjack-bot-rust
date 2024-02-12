use tokio::sync::mpsc::channel;

mod blackjack;
mod card;
mod cli;
mod command;
mod deck;
mod game;
mod player;
mod status;

#[tokio::main]
async fn main() {
    let (tx1, rx1) = channel(32);
    let (tx2, rx2) = channel(32);
    let (tx3, mut rx3) = channel(32);

    let tx1_clone = tx1.clone();
    let tx3_clone = tx3.clone();

    tokio::spawn(async move {
        loop {
            let start_command = rx3.recv().await.unwrap();
            if start_command != "start" {
                continue;
            }
            let _ = cli::start(&tx1_clone, &mut rx3, 1).await;
        }
    });

    tokio::spawn(async move {
        game::play(vec![tx2, tx3], rx1).await;
    });

    cli::run(tx1, rx2, 0, tx3_clone).await;
}
