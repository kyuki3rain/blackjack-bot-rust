use std::io::Write;
use tokio::{io::AsyncWriteExt, sync::mpsc::Receiver};

use crate::utils::message::Message;

pub async fn run(rx: &mut Receiver<Message>) -> Result<(), String> {
    let mut stdout = tokio::io::stdout();

    loop {
        let mut buf = Vec::<u8>::new();
        let message = rx.recv().await.ok_or("Failed to receive message")?;
        writeln!(buf, "{}", message.to_string()).map_err(|e| e.to_string())?;
        stdout.write_all(&buf).await.map_err(|e| e.to_string())?;
    }
}
