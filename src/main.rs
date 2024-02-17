use tokio::sync::mpsc::channel;

mod command_dispatcher;
mod db;
mod game;
mod io;
mod message_dispatcher;
mod utils;

#[tokio::main]
async fn main() {
    let (input_tx, input_rx) = channel(100);
    let (output_tx, mut output_rx) = channel(100);
    let (game_tx, mut game_rx) = channel(100);
    let (message_tx, message_rx) = channel(100);

    let input_tx_clone = input_tx.clone();
    let io_handler = tokio::spawn(async move {
        io::cli::run(&input_tx_clone, &mut output_rx).await.unwrap();
    });

    let game_tx_clone = game_tx.clone();
    let message_tx_clone = message_tx.clone();
    let command_handler = tokio::spawn(async move {
        command_dispatcher::run(input_rx, &game_tx_clone, &message_tx_clone)
            .await
            .unwrap();
    });

    let message_tx_clone = message_tx.clone();
    let game_handler = tokio::spawn(async move {
        game::blackjack::run(&message_tx_clone, &mut game_rx)
            .await
            .unwrap();
    });

    let output_tx_clone = output_tx.clone();
    let message_handler = tokio::spawn(async move {
        message_dispatcher::run(message_rx, vec![output_tx_clone])
            .await
            .unwrap();
    });

    let finished_process = tokio::select! {
        _ = io_handler => "io_handler",
        _ = command_handler => "command_handler",
        _ = game_handler => "game_handler",
        _ = message_handler => "message_handler",
    };

    println!("Finished: {}", finished_process);
}
