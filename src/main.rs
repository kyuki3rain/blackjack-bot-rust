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
    let (tx3, rx3) = channel(32);

    let tx1_clone = tx1.clone();

    tokio::spawn(async move {
        cli::start(tx1_clone, rx3, 1).await;
    });

    tokio::spawn(async move {
        game::play(vec![tx2, tx3], rx1).await;
    });

    cli::run(tx1, rx2, 0).await;
}
