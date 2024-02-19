use std::collections::HashMap;

use crate::game::blackjack::card::Card;

#[derive(Debug, Clone)]
pub enum Message {
    Register(String),
    Rename((String, String)),
    Balance((String, i32)),
    GetBonus((String, i32)),
    Init(u64),
    NoPlayer,
    Start,
    ShowAmounts(Vec<(String, u32)>),
    StartDeliver,
    ShowDealerHand(Vec<Card>, Option<u32>),
    ShowPlayerHand((String, Vec<Card>, u32)),
    DealerBlackjack,
    DealerNotBlackjack,
    WaitPlayer(String),
    PlayerBusted,
    Participate(String),
    Leave(String),
    Bet((String, u32)),
    Hit((String, Card)),
    Stand(String),
    DealerScore(u32),
    ShowResult(HashMap<String, (u32, i32)>),
    Error(String),
}

impl ToString for Message {
    fn to_string(&self) -> String {
        match self {
            Message::Register(name) => format!("{}を登録しました。", name),
            Message::Rename((old_name, new_name)) => {
                format!("{}を{}に変更しました。", old_name, new_name)
            }
            Message::Balance((name, balance)) => format!("{}の残高：{}", name, balance),
            Message::GetBonus((name, amount)) => {
                format!("{}に{}コインのボーナスを付与しました。", name, amount)
            }
            Message::Init(time) => {
                format!(
                    "次のゲームは{}秒後に開始します。掛け金を設定するか、退出してください。",
                    time
                )
            }
            Message::NoPlayer => "プレイヤーがいません。".to_string(),
            Message::Start => "ブラックジャックを開始します。".to_string(),
            Message::ShowAmounts(amounts) => {
                let mut message = "掛け金は以下のようになっています。".to_string();
                for (name, amount) in amounts {
                    message.push('\n');
                    message.push_str(&format!("{}：{}", name, amount));
                }
                message
            }
            Message::StartDeliver => "カードを配ります。".to_string(),
            Message::ShowDealerHand(hand, score) => {
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
            Message::ShowPlayerHand((name, hand, score)) => {
                let hand_str = hand
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{}：{}(score: {})", name, hand_str, score)
            }
            Message::DealerBlackjack => "ディーラーがブラックジャックです。".to_string(),
            Message::DealerNotBlackjack => {
                "ディーラーはブラックジャックではありません。".to_string()
            }
            Message::WaitPlayer(name) => format!("{}のコマンドを待っています。", name),
            Message::PlayerBusted => "バーストしました。".to_string(),
            Message::Participate(name) => format!("{}が参加しました。", name),
            Message::Leave(name) => format!("{}が退室しました。", name),
            Message::Bet((name, amount)) => format!("{}が{}コイン賭けました。", name, amount),
            Message::Hit((name, card)) => {
                format!(
                    "{}がヒットしました。引いたカード：{}",
                    name,
                    card.to_string()
                )
            }
            Message::Stand(name) => format!("{}がスタンドしました。", name),
            Message::DealerScore(score) => format!("ディーラーのスコア：{}", score),
            Message::ShowResult(results) => {
                let mut message = "払い戻しは以下のようになります。".to_string();
                for (name, (amount, diff)) in results {
                    let diff_operator = if *diff > 0 { "+" } else { "" };
                    message.push('\n');
                    message.push_str(&format!("{}：{}({}{})", name, amount, diff_operator, diff));
                }
                message
            }
            Message::Error(error) => error.clone(),
        }
    }
}
