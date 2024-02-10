use crate::blackjack::Status;

mod blackjack;
mod card;
mod deck;
mod player;

fn main() {
    let mut game = blackjack::Blackjack::new();

    game.add_player("Player1".to_string()).unwrap();
    println!("Player1が参加しました。");

    game.add_player("Player2".to_string()).unwrap();
    println!("Player2が参加しました。");

    game.bet("Player1".to_string(), 10).unwrap();
    println!("Player1が10コイン賭けました。");

    game.bet("Player2".to_string(), 20).unwrap();
    println!("Player2が20コイン賭けました。");

    println!();
    println!("ブラックジャックを開始します。");
    println!("掛け金は以下のようになっています。");
    for (name, amount) in game.get_amounts() {
        println!("{}: {}", name, amount);
    }

    println!();
    println!("カードを配ります。");
    let dealer_is_blackjack = game.deal().unwrap();
    print!("{}", game.get_board(true));

    if dealer_is_blackjack {
        println!();
        println!("ディーラーがブラックジャックです。");
        println!("ディーラー：{}", game.get_dealer_hand(false));
    } else {
        println!();
        println!("ディーラーはブラックジャックではありませんでした。");

        while let Status::Playing(player_index) = game.get_status() {
            let player_name = game.get_player_name(player_index);
            println!();
            println!("{}さん、コマンドを入力してください。", player_name);

            let card = game.hit(player_name.to_owned()).unwrap();
            println!(
                "{}さんがヒットしました。引いたカード：{}",
                player_name,
                card.get_symbol()
            );
            println!(
                "{}さんの手札：{}",
                player_name,
                game.get_player_hand(player_name.to_owned()).unwrap()
            );

            let score = game.stand(player_name.to_owned()).unwrap();
            println!("{}さんがスタンドしました。スコア：{}", player_name, score);
        }

        println!();
        println!("ディーラーが操作を開始します。");
        println!("ディーラー：{}", game.get_dealer_hand(false));
        while game.get_status() == Status::DealerTurn {
            game.dealer_hit().unwrap();
            println!("ディーラー：{}", game.get_dealer_hand(false));
        }
        println!("ディーラーのスコアは{}でした。", game.get_dealer_score());
    }

    let result = game.get_result().unwrap();

    println!();
    println!("払い戻しは以下のようになります。");
    for (name, (amount, diff)) in result {
        let diff_operator = if diff > 0 { "+" } else { "" };
        println!("{}: {}（{}{}）", name, amount, diff_operator, diff);
    }
}
