use std::collections::HashMap;

use crate::blackjack::card::Card;

#[derive(Debug, Clone)]
pub struct Message {
    pub to: &'static str,
    pub data: MessageData,
}

impl Message {
    pub fn new(to: &'static str, data: MessageData) -> Self {
        Self { to, data }
    }
}

#[derive(Debug, Clone)]
pub enum MessageData {
    Init(u64),
    NoPlayer,
    Start,
    ShowAmounts(Vec<(String, u32)>),
    StartDeliver,
    ShowDealerHand(Vec<Card>, Option<u32>),
    ShowPlayerHand((String, Vec<Card>, u32)),
    DealerBlackjack,
    DealerNotBlackjack,
    WaitCommand(String),
    PlayerBusted,
    Participate(String),
    Leave(String),
    Bet((String, u32)),
    Hit((String, Card)),
    Stand(String),
    DealerScore(u32),
    Result(HashMap<String, (u32, i32)>),
    Exit,
}

impl ToString for MessageData {
    fn to_string(&self) -> String {
        match self {
            MessageData::Init(time) => {
                format!(
                    "次のゲームは{}秒後に開始します。掛け金を設定するか、退出してください。",
                    time
                )
            }
            MessageData::NoPlayer => "プレイヤーがいません。".to_string(),
            MessageData::Start => "ブラックジャックを開始します。".to_string(),
            MessageData::ShowAmounts(amounts) => {
                let mut message = "掛け金は以下のようになっています。".to_string();
                for (name, amount) in amounts {
                    message.push('\n');
                    message.push_str(&format!("{}：{}", name, amount));
                }
                message
            }
            MessageData::StartDeliver => "カードを配ります。".to_string(),
            MessageData::ShowDealerHand(hand, score) => {
                let hand_str = hand
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");

                if let Some(score) = score {
                    format!("ディーラー：{}(score: {})", hand_str, score)
                } else {
                    format!("ディーラー：{}", hand_str)
                }
            }
            MessageData::ShowPlayerHand((name, hand, score)) => {
                let hand_str = hand
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{}：{}(score: {})", name, hand_str, score)
            }
            MessageData::DealerBlackjack => "ディーラーがブラックジャックです。".to_string(),
            MessageData::DealerNotBlackjack => {
                "ディーラーはブラックジャックではありません。".to_string()
            }
            MessageData::WaitCommand(name) => format!("{}のコマンドを待っています。", name),
            MessageData::PlayerBusted => "バーストしました。".to_string(),
            MessageData::Participate(name) => format!("{}が参加しました。", name),
            MessageData::Leave(name) => format!("{}が退室しました。", name),
            MessageData::Bet((name, amount)) => format!("{}が{}コイン賭けました。", name, amount),
            MessageData::Hit((name, card)) => {
                format!(
                    "{}がヒットしました。引いたカード：{}",
                    name,
                    card.to_string()
                )
            }
            MessageData::Stand(name) => format!("{}がスタンドしました。", name),
            MessageData::DealerScore(score) => format!("ディーラーのスコア：{}", score),
            MessageData::Result(results) => {
                let mut message = "払い戻しは以下のようになります。".to_string();
                for (name, (amount, diff)) in results {
                    let diff_operator = if *diff > 0 { "+" } else { "" };
                    message.push('\n');
                    message.push_str(&format!("{}：{}({}{})", name, amount, diff_operator, diff));
                }
                message
            }
            MessageData::Exit => "ゲームを終了します。".to_string(),
        }
    }
}
