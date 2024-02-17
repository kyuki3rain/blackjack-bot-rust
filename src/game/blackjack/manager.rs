use std::collections::HashMap;

use crate::{
    game::blackjack::{command::GameCommandData, game},
    utils::message::Message,
};
use tokio::sync::mpsc::{Receiver, Sender};

use super::{command::GameCommand, status::Status};

const WAIT_TIME: u64 = 1;

pub async fn manager(
    tx: &Sender<Message>,
    rx: &mut Receiver<GameCommand>,
    initial_players: Vec<String>,
) -> Result<HashMap<String, u32>, String> {
    let mut game = game::Blackjack::new();
    for player in initial_players {
        game.add_player(player.clone())?;
        tx.send(Message::Participate(player)).await.unwrap();
    }

    let (timer_finish_tx, mut timer_finish_rx) = tokio::sync::mpsc::channel(1);
    let timer_handle = tokio::spawn(async move {
        eprintln!("manager: timer start");
        tokio::time::sleep(tokio::time::Duration::from_secs(WAIT_TIME)).await;
        eprintln!("manager: timer finish");
        timer_finish_tx.send(WAIT_TIME).await.unwrap();
    });

    eprintln!("manager: send init");
    tx.send(Message::Init(WAIT_TIME)).await.unwrap();

    loop {
        let result = tokio::select! {
            Some(command) = rx.recv() => {
                eprintln!("manager: command: {:?}", command);
                let user_name = command.user_name;
                match command.data {
                    GameCommandData::Participate => participate(&mut game, tx, user_name).await,
                    GameCommandData::Leave => leave(&mut game, tx, user_name).await,
                    GameCommandData::Hit => hit(&mut game, tx, user_name).await,
                    GameCommandData::Stand => stand(&mut game, tx, user_name).await,
                    GameCommandData::Bet(amount) => bet(&mut game, tx, user_name, amount).await,
                }
            },
            Some(_) = timer_finish_rx.recv() => {
                eprintln!("manager: timer finish");
                start(&mut game, tx).await
            },
        };

        if let Err(e) = result {
            if e == "No player" {
                return Ok(HashMap::new());
            }
            println!("{}", e);
        }

        if let Status::End = game.get_status() {
            break;
        }
    }

    timer_handle.abort();

    let result: HashMap<String, u32> = game
        .get_result()?
        .into_iter()
        .map(|(name, (amount, _))| (name, amount))
        .collect();

    Ok(result)
}

async fn start(game: &mut game::Blackjack, tx: &Sender<Message>) -> Result<(), String> {
    eprintln!("manager: start");
    let amounts = game.get_amounts();
    if amounts.is_empty() {
        tx.send(Message::NoPlayer).await.unwrap();
        return Err("No player".to_string());
    }

    game.start()?;
    tx.send(Message::Start).await.unwrap();
    tx.send(Message::ShowAmounts(amounts)).await.unwrap();
    tx.send(Message::StartDeliver).await.unwrap();

    let dealer_is_blackjack = game.deal()?;
    let dealer_hand = game.get_dealer_hand(true);
    tx.send(Message::ShowDealerHand(dealer_hand, None))
        .await
        .unwrap();
    for (name, hand, score) in game.get_player_hands() {
        tx.send(Message::ShowPlayerHand((name, hand, score)))
            .await
            .unwrap();
    }

    if dealer_is_blackjack {
        tx.send(Message::DealerBlackjack).await.unwrap();
        let dealer_hand = game.get_dealer_hand(false);
        let dealer_score = game.get_dealer_score();
        tx.send(Message::ShowDealerHand(dealer_hand, Some(dealer_score)))
            .await
            .unwrap();
        eprintln!("manager: dealer blackjack");
        next_or_end(game, tx).await?;
        eprintln!("manager: next_or_end");
        return Ok(());
    }

    tx.send(Message::DealerNotBlackjack).await.unwrap();
    next_or_end(game, tx).await?;

    Ok(())
}

async fn participate(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    name: String,
) -> Result<(), String> {
    game.add_player(name.clone())?;
    tx.send(Message::Participate(name)).await.unwrap();
    Ok(())
}

async fn leave(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    name: String,
) -> Result<(), String> {
    game.remove_player(name.clone())?;
    tx.send(Message::Leave(name)).await.unwrap();
    Ok(())
}

async fn hit(game: &mut game::Blackjack, tx: &Sender<Message>, name: String) -> Result<(), String> {
    let (name, card) = game.hit(name.clone())?;
    tx.send(Message::Hit((name.clone(), card))).await.unwrap();

    let (hand, score) = game.get_player_hand(name.clone())?;
    tx.send(Message::ShowPlayerHand((name.clone(), hand, score)))
        .await
        .unwrap();

    if score > 21 {
        game.stand(name.clone())?;
        tx.send(Message::PlayerBusted).await.unwrap();
        next_or_end(game, tx).await?;
    }
    Ok(())
}

async fn stand(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    name: String,
) -> Result<(), String> {
    game.stand(name.clone())?;
    tx.send(Message::Stand(name.clone())).await.unwrap();
    next_or_end(game, tx).await?;
    Ok(())
}

async fn bet(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    name: String,
    amount: u32,
) -> Result<(), String> {
    game.bet(name.clone(), amount)?;
    tx.send(Message::Bet((name.clone(), amount))).await.unwrap();
    Ok(())
}

async fn next_or_end(game: &mut game::Blackjack, tx: &Sender<Message>) -> Result<(), String> {
    loop {
        match game.get_status() {
            Status::Playing(player_index) => {
                let name = game.get_player_name(player_index).clone();
                let (hands, score) = game.get_player_hand(name.clone())?;

                tx.send(Message::ShowPlayerHand((name.clone(), hands, score)))
                    .await
                    .unwrap();
                tx.send(Message::WaitPlayer(name.clone())).await.unwrap();
                break;
            }
            Status::DealerTurn => {
                let hand = game.get_dealer_hand(false);
                tx.send(Message::ShowDealerHand(hand, None)).await.unwrap();
                game.dealer_hit()?;
            }
            Status::End => {
                let score = game.get_dealer_score();
                tx.send(Message::DealerScore(score)).await.unwrap();
                let results = game.get_result()?;
                tx.send(Message::ShowResult(results)).await.unwrap();
                break;
            }
            e => {
                return Err(format!("Invalid status: {:?}", e));
            }
        }
    }

    Ok(())
}
