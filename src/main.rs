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
    let (discord_output_tx, mut discord_output_rx) = channel(100);
    let (cli_output_tx, mut cli_output_rx) = channel(100);
    let (game_tx, mut game_rx) = channel(100);
    let (message_tx, message_rx) = channel(100);

    let discord_input_tx = input_tx.clone();
    let discord_handler = tokio::spawn(async move {
        io::discord::run(discord_input_tx, &mut discord_output_rx)
            .await
            .unwrap();
    });

    let cli_input_tx = input_tx.clone();
    let cli_handler = tokio::spawn(async move {
        io::cli::run(&cli_input_tx, &mut cli_output_rx)
            .await
            .unwrap();
    });

    let game_tx_clone = game_tx.clone();
    let message_tx_clone = message_tx.clone();
    let command_handler: tokio::task::JoinHandle<()> = tokio::spawn(async move {
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

    let discord_output_tx_clone = discord_output_tx.clone();
    let cli_output_tx_clone = cli_output_tx.clone();
    let message_handler = tokio::spawn(async move {
        message_dispatcher::run(
            message_rx,
            vec![discord_output_tx_clone, cli_output_tx_clone],
        )
        .await
        .unwrap();
    });

    let finished_process = tokio::select! {
        _ = cli_handler => "io_handler",
        _ = discord_handler => "discord_handler",
        _ = command_handler => "command_handler",
        _ = game_handler => "game_handler",
        _ = message_handler => "message_handler",
    };

    println!("Finished process: {}", finished_process);
}
