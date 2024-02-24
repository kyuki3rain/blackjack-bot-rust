use tokio::{
    select,
    sync::{broadcast, mpsc},
};

use self::{state::Effect, table::Command};

mod card;
mod deck;
mod player;
pub mod state;
mod status;
pub mod table;

#[derive(Debug)]
pub struct Response {
    pub content: String,
}

pub struct Request {
    res_tx: tokio::sync::oneshot::Sender<Response>,
    command: Command,
}

impl Request {
    pub fn new(res_tx: tokio::sync::oneshot::Sender<Response>, command: Command) -> Self {
        Self { res_tx, command }
    }
}

pub const BETTING_TIME: u64 = 15;

pub async fn run(
    mut game_rx: mpsc::Receiver<Request>,
    broadcast_tx: broadcast::Sender<Effect>,
) -> Result<(), String> {
    let mut players: Vec<String> = vec![];

    loop {
        let mut table = table::Table::new();

        table.init_players(players.clone());
        broadcast_tx
            .send(Effect::Init(table.get_player_order()))
            .unwrap();

        let (start_tx, mut start_rx) = mpsc::channel(1);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(BETTING_TIME)).await;
            start_tx.send(()).await.unwrap();
        });

        loop {
            let request = if table.is_started() {
                game_rx.recv().await.unwrap()
            } else {
                let request = select! {
                    request = game_rx.recv() => request.unwrap(),
                    _ = start_rx.recv() => {
                        if table.get_player_count() == 0 {
                            broadcast_tx.send(Effect::NoPlayer).unwrap();
                            return Ok(());
                        }

                        let effects = table.start()?;
                        for effect in effects {
                            broadcast_tx.send(effect).unwrap();
                        }

                        if table.is_finished() {
                            players = table.get_players();
                            break;
                        }

                        continue;
                    }
                };
                request
            };

            let content = match table.apply_command(request.command.clone()) {
                Ok(effects) => {
                    for effect in effects {
                        broadcast_tx.send(effect).unwrap();
                    }
                    request.command.success_message()
                }
                Err(err) => err,
            };

            let response = Response { content };
            request.res_tx.send(response).unwrap();

            if table.is_finished() {
                broadcast_tx.send(Effect::Finish).unwrap();
                players = table.get_players();
                break;
            }

            if table.is_dealer_turn() {
                let effects = table.dealer_action()?;
                for effect in effects {
                    broadcast_tx.send(effect).unwrap();
                }
                players = table.get_players();
                break;
            }
        }
    }
}
