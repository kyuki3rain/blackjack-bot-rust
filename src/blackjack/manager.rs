use crate::{
    blackjack::game,
    command::{Command, CommandData},
    dispatcher::BROADCAST_ID,
    message::{Message, MessageData},
};
use tokio::sync::mpsc::{Receiver, Sender};

use super::status::Status;

pub async fn manager(tx: Sender<Message>, mut rx: Receiver<Command>) {
    let mut game = game::Blackjack::new();

    loop {
        let command = rx.recv().await.unwrap();

        if let CommandData::Exit = command.data {
            tx.send(Message::new(BROADCAST_ID, MessageData::Exit))
                .await
                .unwrap();
            break;
        }

        if game.get_status() == Status::End {
            if let Err(e) = init(&mut game, &tx, BROADCAST_ID).await {
                println!("{}", e);
                continue;
            }
        }

        let id = command.from;

        let result = match command.data {
            CommandData::Start => start(&mut game, &tx, BROADCAST_ID).await,
            CommandData::Participate(name) => participate(&mut game, &tx, id, name).await,
            CommandData::Leave(name) => leave(&mut game, &tx, id, name).await,
            CommandData::Hit(name) => hit(&mut game, &tx, id, name).await,
            CommandData::Stand(name) => stand(&mut game, &tx, id, name).await,
            CommandData::Bet(name, amount) => bet(&mut game, &tx, id, name, amount).await,
            CommandData::Exit => break,
        };

        if let Err(e) = result {
            println!("{}", e);
        }
    }
}

async fn init(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    id: &'static str,
) -> Result<(), String> {
    game.reset()?;
    tx.send(Message::new(id, MessageData::Init(15)))
        .await
        .unwrap();
    Ok(())
}

async fn start(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    id: &'static str,
) -> Result<(), String> {
    let amounts = game.get_amounts();
    if amounts.is_empty() {
        tx.send(Message::new(id, MessageData::NoPlayer))
            .await
            .unwrap();
        game.finish()?;
        return Ok(());
    }

    game.start()?;
    tx.send(Message::new(id, MessageData::Start)).await.unwrap();
    tx.send(Message::new(id, MessageData::ShowAmounts(amounts)))
        .await
        .unwrap();

    tx.send(Message::new(id, MessageData::StartDeliver))
        .await
        .unwrap();
    let dealer_is_blackjack = game.deal()?;

    tx.send(Message::new(
        id,
        MessageData::ShowDealerHand(game.get_dealer_hand(true), None),
    ))
    .await
    .unwrap();
    for (name, hand, score) in game.get_player_hands() {
        tx.send(Message::new(
            id,
            MessageData::ShowPlayerHand((name, hand, score)),
        ))
        .await
        .unwrap();
    }

    if dealer_is_blackjack {
        tx.send(Message::new(id, MessageData::DealerBlackjack))
            .await
            .unwrap();
        tx.send(Message::new(
            id,
            MessageData::ShowDealerHand(game.get_dealer_hand(false), Some(game.get_dealer_score())),
        ))
        .await
        .unwrap();
        game.finish()?;
        next_or_end(game, tx, id).await?;
        return Ok(());
    }

    tx.send(Message::new(id, MessageData::DealerNotBlackjack))
        .await
        .unwrap();
    next_or_end(game, tx, id).await?;

    Ok(())
}

async fn participate(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    id: &'static str,
    name: String,
) -> Result<(), String> {
    game.add_player(name.clone())?;
    tx.send(Message::new(id, MessageData::Participate(name)))
        .await
        .unwrap();
    Ok(())
}

async fn leave(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    id: &'static str,
    name: String,
) -> Result<(), String> {
    game.remove_player(name.clone())?;
    tx.send(Message::new(id, MessageData::Leave(name)))
        .await
        .unwrap();
    Ok(())
}

async fn hit(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    id: &'static str,
    name: String,
) -> Result<(), String> {
    let (name, card) = game.hit(name.clone())?;
    tx.send(Message::new(id, MessageData::Hit((name.clone(), card))))
        .await
        .unwrap();

    let (hand, score) = game.get_player_hand(name.clone())?;
    tx.send(Message::new(
        id,
        MessageData::ShowPlayerHand((name.clone(), hand, score)),
    ))
    .await
    .unwrap();

    if score > 21 {
        game.stand(name.clone())?;
        tx.send(Message::new(id, MessageData::PlayerBusted))
            .await
            .unwrap();
        next_or_end(game, tx, id).await?;
    }
    Ok(())
}

async fn stand(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    id: &'static str,
    name: String,
) -> Result<(), String> {
    game.stand(name.clone())?;
    tx.send(Message::new(id, MessageData::Stand(name)))
        .await
        .unwrap();
    next_or_end(game, tx, id).await?;
    Ok(())
}

async fn bet(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    id: &'static str,
    name: String,
    amount: u32,
) -> Result<(), String> {
    game.bet(name.clone(), amount)?;
    tx.send(Message::new(id, MessageData::Bet((name, amount))))
        .await
        .unwrap();
    Ok(())
}

async fn next_or_end(
    game: &mut game::Blackjack,
    tx: &Sender<Message>,
    id: &'static str,
) -> Result<(), String> {
    loop {
        match game.get_status() {
            Status::Playing(player_index) => {
                let name = game.get_player_name(player_index).clone();
                let hand = game.get_player_hand(name.clone())?;

                tx.send(Message::new(
                    id,
                    MessageData::ShowPlayerHand((name.clone(), hand.0, hand.1)),
                ))
                .await
                .unwrap();
                tx.send(Message::new(id, MessageData::WaitCommand(name)))
                    .await
                    .unwrap();
                break;
            }
            Status::DealerTurn => {
                let hand = game.get_dealer_hand(false);
                tx.send(Message::new(
                    id,
                    MessageData::ShowDealerHand(hand, Some(game.get_dealer_score())),
                ))
                .await
                .unwrap();
                game.dealer_hit()?;
            }
            Status::End => {
                let score = game.get_dealer_score();
                tx.send(Message::new(id, MessageData::DealerScore(score)))
                    .await
                    .unwrap();
                let results = game.get_result()?;
                tx.send(Message::new(id, MessageData::Result(results)))
                    .await
                    .unwrap();

                game.reset()?;
                tx.send(Message::new(BROADCAST_ID, MessageData::Init(15)))
                    .await
                    .unwrap();
                break;
            }
            e => {
                return Err(format!("Invalid status: {:?}", e));
            }
        }
    }

    Ok(())
}
