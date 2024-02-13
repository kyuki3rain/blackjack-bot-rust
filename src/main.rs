use std::collections::HashMap;

use blackjack::manager;
use cli::{
    display::{self, DISPLAY_ID},
    input,
};
use timer::TIMER_ID;
use tokio::sync::mpsc::channel;

mod blackjack;
mod cli;
mod command;
mod dispatcher;
mod message;
mod timer;

#[tokio::main]
async fn main() {
    let (timer_tx, timer_rx) = channel(1);
    let (manager_tx, manager_rx) = channel(1);
    let (display_tx, display_rx) = channel(1);
    let (dispatcher_tx, dispatcher_rx) = channel(1);

    let manager_tx_clone = manager_tx.clone();

    let mut tx_map = HashMap::new();
    tx_map.insert(TIMER_ID, timer_tx);
    tx_map.insert(DISPLAY_ID, display_tx);

    let dispatcher_handle = tokio::spawn(async move {
        dispatcher::dispatcher(tx_map, dispatcher_rx).await;
    });

    let timer_handle = tokio::spawn(async move {
        timer::timer(manager_tx_clone, timer_rx).await;
    });

    let display_handle = tokio::spawn(async move {
        display::display(display_rx).await;
    });

    let input_listener_handle = tokio::spawn(async move {
        input::input_listener(manager_tx).await;
    });

    manager::manager(dispatcher_tx, manager_rx).await;

    timer_handle.abort();
    input_listener_handle.await.unwrap();
    dispatcher_handle.await.unwrap();
    display_handle.await.unwrap();
}
