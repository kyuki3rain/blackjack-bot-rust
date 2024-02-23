use tokio::sync::{broadcast, mpsc};

pub enum Command {
    Ping(String),
}

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

#[derive(Debug, Clone)]
pub struct Effect {
    pub content: String,
}

pub async fn run(
    mut game_rx: mpsc::Receiver<Request>,
    broadcast_tx: broadcast::Sender<Effect>,
) -> Result<(), String> {
    loop {
        let request = game_rx.recv().await.unwrap();
        let content = match request.command {
            Command::Ping(name) => format!("{} さん、こんにちは", name),
        };
        let response = Response { content };
        request.res_tx.send(response).unwrap();

        for i in 0..3 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            broadcast_tx
                .send(Effect {
                    content: format!("test message: {}", i),
                })
                .unwrap();
        }
    }
}
