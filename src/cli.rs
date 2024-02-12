use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    command::{Command, CommandResult, CommandResultType, CommandType},
    status::Status,
};

async fn exec(
    id: usize,
    content: CommandType,
    tx: &Sender<String>,
    rx: &mut Receiver<String>,
) -> Result<CommandResultType, String> {
    let type_string = content.to_type_string();
    let command = Command::new(id, content);
    tx.send(command.to_string()).await.unwrap();
    let response = rx.recv().await.unwrap();
    let result: CommandResult = response.parse().unwrap_or_else(|_| {
        panic!("{}のパースに失敗しました。", response);
    });
    if result.check_result_type(command.content.unwrap()) {
        Ok(result.content.unwrap())
    } else {
        Err(format!("{}に失敗しました。", type_string))
    }
}

pub async fn start(
    tx: &Sender<String>,
    rx: &mut Receiver<String>,
    id: usize,
) -> Result<(), String> {
    exec(id, CommandType::Reset, tx, rx).await?;
    println!("次のゲームは15秒後に開始します。掛け金を設定するか、退出してください。");
    tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

    let result = exec(id, CommandType::GetAmounts, tx, rx).await?;
    if let CommandResultType::GetAmounts(amounts) = result {
        if amounts.is_empty() {
            println!("参加者がいません。");
            exec(id, CommandType::Finish, tx, rx).await?;
            return Ok(());
        }

        exec(id, CommandType::Start, tx, rx).await?;
        println!("ブラックジャックを開始します。");
        println!("掛け金は以下のようになっています。");
        for (name, amount) in amounts {
            println!("{}: {}", name, amount);
        }
    };

    println!();
    println!("カードを配ります。");
    let result = exec(id, CommandType::Deal, tx, rx).await?;
    let dealer_is_blackjack = if let CommandResultType::Deal(dealer_is_blackjack) = result {
        dealer_is_blackjack
    } else {
        false
    };

    let result = exec(id, CommandType::GetBoard(true), tx, rx).await?;
    if let CommandResultType::GetBoard(board) = result {
        print!("{}", board);
    }

    println!();
    if dealer_is_blackjack {
        println!("ディーラーがブラックジャックです。");
        let result = exec(id, CommandType::GetDealerHand(false), tx, rx).await?;
        if let CommandResultType::GetDealerHand(hand) = result {
            println!("ディーラー：{}", hand);
        }
        exec(id, CommandType::Finish, tx, rx).await?;
    } else {
        println!("ディーラーはブラックジャックではありませんでした。");
        next_or_end(tx, rx, id, tx).await?;
    }

    Ok(())
}

pub async fn run(
    tx: Sender<String>,
    mut rx: Receiver<String>,
    id: usize,
    start_tx: Sender<String>,
) {
    let input = std::io::stdin();

    loop {
        let mut line = String::new();
        input.read_line(&mut line).unwrap();
        let line = line.trim();
        if line == "/exit" {
            break;
        }

        match exec(id, CommandType::GetStatus, &tx, &mut rx).await {
            Ok(CommandResultType::GetStatus(status)) => {
                if status == Status::End {
                    start_tx.send("start".to_string()).await.unwrap();
                }
            }
            Err(e) => {
                println!("{}", e);
            }
            _ => {
                println!("ゲームの状態の取得に失敗しました。");
            }
        }

        if let Err(e) = exec_user_command(line, id, &tx, &mut rx, &start_tx).await {
            println!("{}", e);
        }
    }
}

async fn exec_user_command(
    line: &str,
    id: usize,
    tx: &Sender<String>,
    rx: &mut Receiver<String>,
    start_tx: &Sender<String>,
) -> Result<(), String> {
    let mut iter = line.split_whitespace();

    // unwrap or Err
    let content_str = if let Some(content_str) = iter.next() {
        content_str
    } else {
        return Err("コマンドを入力してください。".to_string());
    };

    let name = if let Some(name) = iter.next() {
        name
    } else {
        return Err("名前を入力してください。".to_string());
    };

    let content = match content_str {
        "/participate" => CommandType::Participate(name.to_string()),
        "/leave" => CommandType::Leave(name.to_string()),
        "/bet" => {
            let amount: u32 = if let Some(amount_str) = iter.next() {
                if let Ok(amount) = amount_str.parse() {
                    amount
                } else {
                    return Err("金額を入力してください。".to_string());
                }
            } else {
                return Err("金額を入力してください。".to_string());
            };
            CommandType::Bet(name.to_string(), amount)
        }
        "/hit" => CommandType::Hit(name.to_string()),
        "/stand" => CommandType::Stand(name.to_string()),
        _ => return Err("不正なコマンドです。".to_string()),
    };

    let result = exec(id, content, tx, rx).await?;

    match result {
        CommandResultType::Participate(name) => println!("{}が参加しました。", name),
        CommandResultType::Leave(name) => println!("{}が退室しました。", name),

        CommandResultType::Bet((name, amount)) => {
            println!("{}が{}コイン賭けました。", name, amount)
        }
        CommandResultType::Hit((name, card)) => {
            println!(
                "{}がヒットしました。引いたカード：{}",
                name,
                card.to_string()
            );
            let command = CommandType::GetPlayerHand(name.to_string());
            let result = exec(id, command, tx, rx).await?;
            if let CommandResultType::GetPlayerHand((hand, score)) = result {
                println!("{}：{}(score: {})", name, hand, score);
            } else {
                println!("プレイヤーの手札の取得に失敗しました。");
            }
        }
        CommandResultType::Stand((name, score)) => {
            println!("{}がスタンドしました。スコア：{}", name, score);
            next_or_end(tx, rx, id, start_tx).await?;
        }
        e => {
            println!("unexpected content {:?}", e);
        }
    }

    Ok(())
}

async fn next_or_end(
    tx: &Sender<String>,
    rx: &mut Receiver<String>,
    id: usize,
    start_tx: &Sender<String>,
) -> Result<(), String> {
    loop {
        let result = exec(id, CommandType::GetStatus, tx, rx).await?;

        if let CommandResultType::GetStatus(status) = result {
            match status {
                Status::Playing(player_index) => {
                    let result = exec(id, CommandType::GetPlayerName(player_index), tx, rx).await?;
                    if let CommandResultType::GetPlayerName(name) = result {
                        println!();
                        let result =
                            exec(id, CommandType::GetPlayerHand(name.to_owned()), tx, rx).await?;
                        if let CommandResultType::GetPlayerHand((hand, score)) = result {
                            println!("{}：{} ({})", name.to_owned(), hand, score);
                        }
                        println!("{}さん、コマンドを入力してください。", name);
                    }
                    break;
                }
                Status::DealerTurn => {
                    let result = exec(id, CommandType::DealerHit, tx, rx).await?;
                    if let CommandResultType::DealerHit(_) = result {
                        let result = exec(id, CommandType::GetDealerHand(false), tx, rx).await?;
                        if let CommandResultType::GetDealerHand(hand) = result {
                            println!("ディーラー：{}", hand);
                        }
                    }
                }
                Status::End => {
                    let result = exec(id, CommandType::GetDealerScore, tx, rx).await?;
                    if let CommandResultType::GetDealerScore(score) = result {
                        println!("ディーラーのスコアは{}でした。", score);
                    }
                    println!();
                    println!("払い戻しは以下のようになります。");
                    let result = exec(id, CommandType::GetResult, tx, rx).await?;
                    if let CommandResultType::GetResult(result) = result {
                        for (name, (amount, diff)) in result {
                            let diff_operator = if diff > 0 { "+" } else { "" };
                            println!("{}: {}（{}{}）", name, amount, diff_operator, diff);
                        }
                    }
                    start_tx.send("start".to_string()).await.unwrap();
                    break;
                }
                e => {
                    println!("unexpected status {:?}", e);
                    break;
                }
            }
        }
    }

    Ok(())
}
