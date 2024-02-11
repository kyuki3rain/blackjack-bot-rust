use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    command::{Command, CommandResult, CommandResultType},
    status::Status,
};

pub async fn run(tx: Sender<String>, mut rx: Receiver<String>, id: usize) {
    let input = std::io::stdin();

    loop {
        let mut line = String::new();
        input.read_line(&mut line).unwrap();
        let line = line.trim();

        if line == "/exit" {
            break;
        }

        let mut iter = line.split_whitespace();

        // Noneならcontinueし、Someなら外して次の処理に進む
        let content_str = if let Some(content_str) = iter.next() {
            content_str
        } else {
            continue;
        };

        let name = if let Some(name) = iter.next() {
            name
        } else {
            println!("名前を入力してください。");
            continue;
        };

        let content = match content_str {
            "/participate" => Ok(crate::command::CommandType::Participate(name.to_string())),
            "/leave" => Ok(crate::command::CommandType::Leave(name.to_string())),
            "/bet" => {
                let amount = if let Some(amount_str) = iter.next() {
                    if let Ok(amount) = amount_str.parse() {
                        amount
                    } else {
                        println!("金額は数字で入力してください。");
                        continue;
                    }
                } else {
                    println!("金額を入力してください。");
                    continue;
                };
                Ok(crate::command::CommandType::Bet(name.to_string(), amount))
            }
            "/hit" => Ok(crate::command::CommandType::Hit(name.to_string())),
            "/stand" => Ok(crate::command::CommandType::Stand(name.to_string())),
            _ => {
                println!("不明なコマンドです。");
                continue;
            }
        };

        let command = Command { content, from: id };
        tx.send(command.to_string()).await.unwrap();
        let result_str = rx.recv().await.unwrap();

        let result: CommandResult = if let Ok(result) = result_str.parse() {
            result
        } else {
            println!("unexpected result: {}", result_str);
            continue;
        };

        match result.content {
            Ok(content) => match content {
                CommandResultType::Participate(name) => {
                    println!("{}が参加しました。", name);
                }
                CommandResultType::Leave(name) => {
                    println!("{}が退室しました。", name);
                }
                CommandResultType::Bet((name, amount)) => {
                    println!("{}が{}コイン賭けました。", name, amount);
                }
                CommandResultType::Hit((name, card)) => {
                    println!(
                        "{}がヒットしました。引いたカード：{}",
                        name,
                        card.to_string()
                    );
                    show_player_hand(&name, &tx, &mut rx, id).await;
                }
                CommandResultType::Stand((name, score)) => {
                    println!("{}がスタンドしました。スコア：{}", name, score);
                    next_or_end(&tx, &mut rx, id).await;
                }
                e => {
                    println!("unexpected content {:?}", e);
                }
            },
            Err(_) => todo!(),
        }
    }
}

async fn next_or_end(tx: &Sender<String>, rx: &mut Receiver<String>, id: usize) {
    loop {
        match get_satus(tx, rx, id).await {
            Ok(Status::Playing(player_index)) => {
                let player_name = get_player_name(player_index, tx, rx, id).await.unwrap();
                println!();
                println!("{}さん、コマンドを入力してください。", player_name);
                break;
            }
            Ok(Status::DealerTurn) => {
                dealer_hit(tx, rx, id).await;
            }
            Ok(Status::End) => {
                get_dealer_score(tx, rx, id).await;
                println!();
                println!("払い戻しは以下のようになります。");
                get_result(tx, rx, id).await;
                break;
            }
            e => {
                println!("unexpected status {:?}", e);
                break;
            }
        }
    }
}

async fn exec(command: Command, tx: &Sender<String>, rx: &mut Receiver<String>) -> CommandResult {
    tx.send(command.to_string()).await.unwrap();
    let response = rx.recv().await.unwrap();
    response.parse().unwrap()
}

pub async fn start(tx: Sender<String>, mut rx: Receiver<String>, id: usize) {
    tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

    let content = Ok(crate::command::CommandType::Start);
    let result = exec(Command { content, from: id }, &tx, &mut rx).await;
    if let Ok(CommandResultType::Start(_)) = result.content {
        println!("ブラックジャックを開始します。");
        println!("掛け金は以下のようになっています。");
        get_amounts(&tx, &mut rx, id).await;
        println!();
        println!("カードを配ります。");
        let dealer_is_blackjack = deal(&tx, &mut rx, id).await.unwrap();
        get_board(true, &tx, &mut rx, id).await;
        println!();
        if dealer_is_blackjack {
            println!("ディーラーがブラックジャックです。");
            show_dealer_hand(false, &tx, &mut rx, id).await;
        } else {
            println!("ディーラーはブラックジャックではありませんでした。");
            next_or_end(&tx, &mut rx, id).await;
        }
    } else {
        println!("開始に失敗しました。");
    }
}

async fn get_amounts(tx: &Sender<String>, rx: &mut Receiver<String>, id: usize) {
    let content = Ok(crate::command::CommandType::GetAmounts);
    let result = exec(Command { content, from: id }, tx, rx).await;
    if let Ok(CommandResultType::GetAmounts(amounts)) = result.content {
        for (name, amount) in amounts {
            println!("{}: {}", name, amount);
        }
    } else {
        println!("掛け金の取得に失敗しました。");
    }
}

async fn get_satus(
    tx: &Sender<String>,
    rx: &mut Receiver<String>,
    id: usize,
) -> Result<Status, ()> {
    let content = Ok(crate::command::CommandType::GetStatus);
    let result = exec(Command { content, from: id }, tx, rx).await;
    if let Ok(CommandResultType::GetStatus(status)) = result.content {
        Ok(status)
    } else {
        Err(())
    }
}

async fn deal(tx: &Sender<String>, rx: &mut Receiver<String>, id: usize) -> Result<bool, ()> {
    let content = Ok(crate::command::CommandType::Deal);
    let result = exec(Command { content, from: id }, tx, rx).await;
    if let Ok(CommandResultType::Deal(dealer_is_blackjack)) = result.content {
        Ok(dealer_is_blackjack)
    } else {
        println!("カードの配布に失敗しました。");
        Err(())
    }
}

async fn show_dealer_hand(
    hide_first: bool,
    tx: &Sender<String>,
    rx: &mut Receiver<String>,
    id: usize,
) {
    let content = Ok(crate::command::CommandType::GetDealerHand(hide_first));
    let result = exec(Command { content, from: id }, tx, rx).await;
    if let Ok(CommandResultType::GetDealerHand(hand)) = result.content {
        println!("ディーラー：{}", hand);
    } else {
        println!("ディーラーの手札の取得に失敗しました。");
    }
}

async fn get_player_name(
    index: usize,
    tx: &Sender<String>,
    rx: &mut Receiver<String>,
    id: usize,
) -> Result<String, ()> {
    let content = Ok(crate::command::CommandType::GetPlayerName(index));
    let result = exec(Command { content, from: id }, tx, rx).await;
    if let Ok(CommandResultType::GetPlayerName(name)) = result.content {
        Ok(name)
    } else {
        println!("プレイヤー名の取得に失敗しました。");
        Err(())
    }
}

async fn get_board(hide_first: bool, tx: &Sender<String>, rx: &mut Receiver<String>, id: usize) {
    let content = Ok(crate::command::CommandType::GetBoard(hide_first));
    let result = exec(Command { content, from: id }, tx, rx).await;
    if let Ok(CommandResultType::GetBoard(board)) = result.content {
        print!("{}", board);
    } else {
        println!("ボードの取得に失敗しました。");
    }
}

async fn show_player_hand(name: &str, tx: &Sender<String>, rx: &mut Receiver<String>, id: usize) {
    let content = Ok(crate::command::CommandType::GetPlayerHand(name.to_string()));
    let result = exec(Command { content, from: id }, tx, rx).await;
    if let Ok(CommandResultType::GetPlayerHand(hand)) = result.content {
        println!("{}：{}", name, hand);
    } else {
        println!("プレイヤーの手札の取得に失敗しました。");
    }
}

async fn dealer_hit(tx: &Sender<String>, rx: &mut Receiver<String>, id: usize) {
    let content = Ok(crate::command::CommandType::DealerHit);
    let result = exec(Command { content, from: id }, tx, rx).await;
    if let Ok(CommandResultType::DealerHit(_)) = result.content {
        show_dealer_hand(false, tx, rx, id).await;
    } else {
        println!("ディーラーのヒットに失敗しました。");
    }
}

async fn get_dealer_score(tx: &Sender<String>, rx: &mut Receiver<String>, id: usize) {
    let content = Ok(crate::command::CommandType::GetDealerScore);
    let result = exec(Command { content, from: id }, tx, rx).await;
    if let Ok(CommandResultType::GetDealerScore(score)) = result.content {
        println!("ディーラーのスコアは{}でした。", score);
    } else {
        println!("ディーラーのスコアの取得に失敗しました。");
    }
}

async fn get_result(tx: &Sender<String>, rx: &mut Receiver<String>, id: usize) {
    let content = Ok(crate::command::CommandType::GetResult);
    let result = exec(Command { content, from: id }, tx, rx).await;
    if let Ok(CommandResultType::GetResult(result)) = result.content {
        for (name, (amount, diff)) in result {
            let diff_operator = if diff > 0 { "+" } else { "" };
            println!("{}: {}（{}{}）", name, amount, diff_operator, diff);
        }
    } else {
        println!("結果の取得に失敗しました。");
    }
}
