use std::str::FromStr;

use std::{collections::HashMap, num::ParseIntError, str::ParseBoolError};

use crate::card::Card;
use crate::status::Status;

#[derive(Debug)]
pub struct Command {
    pub from: usize,
    pub content: Result<CommandType, String>,
}

#[derive(Debug)]
pub enum CommandType {
    Participate(String),
    Leave(String),
    Start,
    Bet(String, u32),
    Hit(String),
    Stand(String),
    Deal,
    DealerHit,
    GetAmounts,
    GetStatus,
    GetPlayerName(usize),
    GetBoard(bool),
    GetPlayerHand(String),
    GetDealerHand(bool),
    GetDealerScore,
    GetResult,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace();

        let from = iter
            .next()
            .ok_or("From not found")?
            .parse()
            .map_err(|e: ParseIntError| e.to_string())?;

        let content = match iter.next() {
            Some("/participate") => {
                let name = iter.next().ok_or("Name not found")?;
                Ok(CommandType::Participate(name.to_string()))
            }
            Some("/leave") => {
                let name = iter.next().ok_or("Name not found")?;
                Ok(CommandType::Leave(name.to_string()))
            }
            Some("/start") => Ok(CommandType::Start),
            Some("/bet") => {
                let name = iter.next().ok_or("Name not found")?;
                let amount: u32 = iter
                    .next()
                    .ok_or("Amount not found")?
                    .parse()
                    .map_err(|e: ParseIntError| e.to_string())?;
                Ok(CommandType::Bet(name.to_string(), amount))
            }
            Some("/hit") => {
                let name = iter.next().ok_or("Name not found")?;
                Ok(CommandType::Hit(name.to_string()))
            }
            Some("/stand") => {
                let name = iter.next().ok_or("Name not found")?;
                Ok(CommandType::Stand(name.to_string()))
            }
            Some("/deal") => Ok(CommandType::Deal),
            Some("/dealer_hit") => Ok(CommandType::DealerHit),
            Some("/get_amounts") => Ok(CommandType::GetAmounts),
            Some("/get_status") => Ok(CommandType::GetStatus),
            Some("/get_player_name") => {
                let index = iter
                    .next()
                    .ok_or("Index not found")?
                    .parse()
                    .map_err(|e: ParseIntError| e.to_string())?;
                Ok(CommandType::GetPlayerName(index))
            }
            Some("/get_board") => {
                let hide_first = iter
                    .next()
                    .ok_or("Hide first not found")?
                    .parse()
                    .map_err(|e: ParseBoolError| e.to_string())?;
                Ok(CommandType::GetBoard(hide_first))
            }
            Some("/get_player_hand") => {
                let name = iter.next().ok_or("Name not found")?;
                Ok(CommandType::GetPlayerHand(name.to_string()))
            }
            Some("/get_dealer_hand") => {
                let hide_first = iter
                    .next()
                    .ok_or("Hide first not found")?
                    .parse()
                    .map_err(|e: ParseBoolError| e.to_string())?;
                Ok(CommandType::GetDealerHand(hide_first))
            }
            Some("/get_dealer_score") => Ok(CommandType::GetDealerScore),
            Some("/get_result") => Ok(CommandType::GetResult),
            _ => Err("Invalid command".to_string()),
        };

        Ok(Command { content, from })
    }
}

impl ToString for Command {
    fn to_string(&self) -> String {
        let content = match &self.content {
            Ok(content) => content,
            Err(_) => return "Invalid command".to_string(),
        };

        let mut formatted = format!("{} ", self.from);

        let content = match content {
            CommandType::Participate(name) => format!("/participate {}", name),
            CommandType::Leave(name) => format!("/leave {}", name),
            CommandType::Start => "/start".to_string(),
            CommandType::Bet(name, amount) => format!("/bet {} {}", name, amount),
            CommandType::Hit(name) => format!("/hit {}", name),
            CommandType::Stand(name) => format!("/stand {}", name),
            CommandType::Deal => "/deal".to_string(),
            CommandType::DealerHit => "/dealer_hit".to_string(),
            CommandType::GetAmounts => "/get_amounts".to_string(),
            CommandType::GetStatus => "/get_status".to_string(),
            CommandType::GetPlayerName(index) => format!("/get_player_name {}", index),
            CommandType::GetBoard(hide_first) => format!("/get_board {}", hide_first),
            CommandType::GetPlayerHand(name) => format!("/get_player_hand {}", name),
            CommandType::GetDealerHand(hide_first) => format!("/get_dealer_hand {}", hide_first),
            CommandType::GetDealerScore => "/get_dealer_score".to_string(),
            CommandType::GetResult => "/get_result".to_string(),
        };
        formatted += &content;

        formatted
    }
}

#[derive(Debug)]
pub struct CommandResult {
    pub content: Result<CommandResultType, String>,
    pub to: usize,
}

#[derive(Debug)]
pub enum CommandResultType {
    Participate(String),
    Leave(String),
    Start(()),
    Bet((String, u32)),
    Hit((String, Card)),
    Stand((String, u32)),
    Deal(bool),
    DealerHit(Option<Card>),
    GetAmounts(Vec<(String, u32)>),
    GetStatus(Status),
    GetPlayerName(String),
    GetBoard(String),
    GetPlayerHand(String),
    GetDealerHand(String),
    GetDealerScore(u32),
    GetResult(HashMap<String, (u32, i32)>),
}

impl FromStr for CommandResult {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.splitn(3, ' ');

        let to = iter
            .next()
            .ok_or("To not found")?
            .parse()
            .map_err(|e: ParseIntError| e.to_string())?;

        let content = match iter.next() {
            Some("/participate") => {
                let name = iter.next().ok_or("Name not found")?;
                Ok(CommandResultType::Participate(name.to_string()))
            }
            Some("/leave") => {
                let name = iter.next().ok_or("Name not found")?;
                Ok(CommandResultType::Leave(name.to_string()))
            }
            Some("/start") => Ok(CommandResultType::Start(())),
            Some("/bet") => {
                let mut iter = iter.next().unwrap().split_whitespace();
                let name = iter.next().ok_or("Name not found")?;
                let amount = iter
                    .next()
                    .ok_or("Amount not found")?
                    .parse()
                    .map_err(|e: ParseIntError| e.to_string())?;
                Ok(CommandResultType::Bet((name.to_string(), amount)))
            }
            Some("/hit") => {
                let mut iter = iter.next().unwrap().split_whitespace();
                let name = iter.next().ok_or("Name not found")?;
                let card = iter.next().ok_or("Card not found")?.parse()?;
                Ok(CommandResultType::Hit((name.to_string(), card)))
            }
            Some("/stand") => {
                let mut iter = iter.next().unwrap().split_whitespace();
                let name = iter.next().ok_or("Name not found")?;
                let score = iter
                    .next()
                    .ok_or("Score not found")?
                    .parse()
                    .map_err(|e: ParseIntError| e.to_string())?;
                Ok(CommandResultType::Stand((name.to_string(), score)))
            }
            Some("/deal") => {
                let dealer_is_blackjack = iter
                    .next()
                    .ok_or("Dealer is blackjack not found")?
                    .parse()
                    .map_err(|e: ParseBoolError| e.to_string())?;
                Ok(CommandResultType::Deal(dealer_is_blackjack))
            }
            Some("/dealer_hit") => {
                let card = iter.next().map(Card::from_str).transpose()?;
                Ok(CommandResultType::DealerHit(card))
            }
            Some("/get_amounts") => {
                let mut amounts = vec![];
                for pair in iter.next().unwrap().split_whitespace() {
                    let mut pair_iter = pair.split(':');
                    let name = pair_iter.next().ok_or("Name not found")?;
                    let amount = pair_iter
                        .next()
                        .ok_or("Amount not found")?
                        .parse()
                        .map_err(|e: ParseIntError| e.to_string())?;
                    amounts.push((name.to_string(), amount));
                }
                Ok(CommandResultType::GetAmounts(amounts))
            }
            Some("/get_status") => {
                let status = iter.next().ok_or("Status not found")?.parse()?;
                Ok(CommandResultType::GetStatus(status))
            }
            Some("/get_player_name") => {
                let name = iter.next().ok_or("Name not found")?;
                Ok(CommandResultType::GetPlayerName(name.to_string()))
            }
            Some("/get_board") => {
                let board = iter.next().ok_or("Board not found")?;
                Ok(CommandResultType::GetBoard(board.to_string()))
            }
            Some("/get_player_hand") => {
                let hand = iter.next().ok_or("Hand not found")?;
                Ok(CommandResultType::GetPlayerHand(hand.to_string()))
            }
            Some("/get_dealer_hand") => {
                let hand = iter.next().ok_or("Hand not found")?;
                Ok(CommandResultType::GetDealerHand(hand.to_string()))
            }
            Some("/get_dealer_score") => {
                let score = iter
                    .next()
                    .ok_or("Score not found")?
                    .parse()
                    .map_err(|e: ParseIntError| e.to_string())?;
                Ok(CommandResultType::GetDealerScore(score))
            }
            Some("/get_result") => {
                let mut result = HashMap::new();
                for pair in iter.next().ok_or("Result not found")?.split_whitespace() {
                    let mut pair_iter = pair.split(':');
                    let name = pair_iter.next().ok_or("Name not found")?;
                    let mut pair_iter = pair_iter.next().ok_or("Pair not found")?.split(',');
                    let amount = pair_iter
                        .next()
                        .ok_or("Amount not found")?
                        .parse()
                        .map_err(|e: ParseIntError| e.to_string())?;
                    let score = pair_iter
                        .next()
                        .ok_or("Score not found")?
                        .parse()
                        .map_err(|e: ParseIntError| e.to_string())?;
                    result.insert(name.to_string(), (amount, score));
                }
                Ok(CommandResultType::GetResult(result))
            }
            _ => Err("Invalid command".to_string()),
        };

        Ok(CommandResult { content, to })
    }
}

impl ToString for CommandResult {
    fn to_string(&self) -> String {
        let content = match &self.content {
            Ok(content) => content,
            Err(_) => return "Invalid command".to_string(),
        };

        let mut formatted = format!("{} ", self.to);

        let content = match content {
            CommandResultType::Participate(name) => format!("/participate {}", name),
            CommandResultType::Leave(name) => format!("/leave {}", name),
            CommandResultType::Start(()) => "/start".to_string(),
            CommandResultType::Bet((name, amount)) => format!("/bet {} {}", name, amount),
            CommandResultType::Hit((name, card)) => {
                let card = card.to_string();
                format!("/hit {} {}", name, card)
            }
            CommandResultType::Stand((name, score)) => format!("/stand {} {}", name, score),
            CommandResultType::Deal(dealer_is_blackjack) => {
                format!("/deal {}", dealer_is_blackjack)
            }
            CommandResultType::DealerHit(card) => {
                let card = card
                    .map(|card| card.to_string())
                    .unwrap_or("None".to_string());
                format!("/dealer_hit {}", card)
            }
            CommandResultType::GetAmounts(amounts) => {
                let mut formatted = "/get_amounts".to_string();
                for (name, amount) in amounts {
                    formatted += &format!(" {}:{} ", name, amount);
                }
                formatted
            }
            CommandResultType::GetStatus(status) => format!("/get_status {}", status.to_string()),
            CommandResultType::GetPlayerName(name) => format!("/get_player_name {}", name),
            CommandResultType::GetBoard(board) => format!("/get_board {}", board),
            CommandResultType::GetPlayerHand(hand) => format!("/get_player_hand {}", hand),
            CommandResultType::GetDealerHand(hand) => format!("/get_dealer_hand {}", hand),
            CommandResultType::GetDealerScore(score) => format!("/get_dealer_score {}", score),
            CommandResultType::GetResult(result) => {
                let mut formatted = "/get_result".to_string();
                for (name, (amount, score)) in result {
                    formatted += &format!(" {}:{},{} ", name, amount, score);
                }
                formatted
            }
        };
        formatted += &content;

        formatted
    }
}
