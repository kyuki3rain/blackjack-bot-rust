use std::env;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use database::{
    bet, create_discord_user, create_table, delete_table, get_balance, get_table_id,
    get_username_by_discord, UserId,
};
use dotenvy::dotenv;
use game::state::{self, Effect};
use game::table::Command;
use game::{Request, Response, BETTING_TIME};
use serenity::all::ChannelId;
use serenity::async_trait;
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::client::{Context, EventHandler};
use serenity::model::prelude::{CommandOptionType, GatewayIntents, Interaction, Ready};
use sqlx::{Pool, Postgres};
use tokio::sync::{broadcast, mpsc};

use crate::database::save_result;

mod database;
mod game;

pub struct Handler {
    game_txs: Arc<Mutex<HashMap<i32, tokio::sync::mpsc::Sender<Request>>>>,
    broadcast_txs: Arc<Mutex<HashMap<i32, tokio::sync::broadcast::Sender<Effect>>>>,
    remove_table_tx: tokio::sync::mpsc::Sender<i32>,
    conn: Pool<Postgres>,
}

pub async fn exec_game_command(
    game_tx: tokio::sync::mpsc::Sender<Request>,
    command: Command,
) -> String {
    let (res_tx, res_rx) = tokio::sync::oneshot::channel::<Response>();

    let request = Request::new(res_tx, command);

    game_tx.send(request).await.unwrap();
    res_rx.await.unwrap().content
}

impl Handler {
    async fn start(
        &self,
        http: Arc<serenity::http::Http>,
        channel_id: ChannelId,
    ) -> Result<CreateInteractionResponseMessage, String> {
        let channel_id_u64 = channel_id.get();

        create_table(&self.conn, channel_id_u64)
            .await
            .map_err(|_| "登録に失敗しました".to_string())?;

        let table_id = get_table_id(&self.conn, channel_id_u64)
            .await
            .map_err(|_| "テーブルIDの取得に失敗しました".to_string())?;
        let (game_tx, game_rx) = tokio::sync::mpsc::channel(1);
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.game_txs.lock().unwrap().entry(table_id)
        {
            e.insert(game_tx);
        } else {
            return Err("このチャンネルには既にゲームが登録されています".to_string());
        };

        let (broadcast_tx, mut broadcast_rx) = broadcast::channel(100);
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.broadcast_txs.lock().unwrap().entry(table_id)
        {
            e.insert(broadcast_tx.clone());
        } else {
            return Err("このチャンネルには既にゲームが登録されています".to_string());
        };
        {
            let broadcast_tx = broadcast_tx.clone();
            tokio::spawn(game::run(game_rx, broadcast_tx.clone()));
        }

        {
            let remove_table_tx = self.remove_table_tx.clone();
            let conn = self.conn.clone();
            tokio::spawn(async move {
                let mut state = state::State::new();
                loop {
                    let effect = broadcast_rx.recv().await.unwrap();
                    state.apply_effect(effect.clone());

                    match effect {
                        Effect::Init(player_order) => {
                            channel_id.say(&http, format!("{}秒後に次のゲームを始めます。参加・退室・ベットをしてください。", BETTING_TIME)).await.unwrap();
                            channel_id
                                .say(&http, format!("現在の参加者: {}", player_order.join(", ")))
                                .await
                                .unwrap();
                        }
                        Effect::AddPlayer(_) => {}
                        Effect::RemovePlayer(_) => {}
                        Effect::Bet(name, amount) => {
                            bet(&conn, UserId::Name(name), amount as i32).await.unwrap()
                        }
                        Effect::Deal(_, _) => {
                            channel_id.say(&http, "カードを配布します。").await.unwrap();
                            channel_id.say(&http, state.to_string()).await.unwrap();
                        }
                        Effect::DealerBlackjack => {
                            channel_id
                                .say(&http, "ディーラーがブラックジャックです。")
                                .await
                                .unwrap();
                        }
                        Effect::Start => {
                            channel_id.say(&http, "ゲームを開始します。").await.unwrap();
                            channel_id
                                .say(&http, "掛け金は以下のようになっています。")
                                .await
                                .unwrap();
                            channel_id
                                .say(
                                    &http,
                                    state
                                        .get_amounts()
                                        .iter()
                                        .map(|(name, amount)| format!("{}: {}", name, amount))
                                        .collect::<Vec<_>>()
                                        .join("\n"),
                                )
                                .await
                                .unwrap();
                        }
                        Effect::AddCard(name, _) => {
                            let player = state.get_player(&name).unwrap();
                            channel_id.say(&http, format!("{}", player)).await.unwrap();
                        }
                        Effect::AddDealerCard(_) => {
                            channel_id
                                .say(&http, format!("{}", state.get_dealer()))
                                .await
                                .unwrap();
                        }
                        Effect::OpenDealerCard(_) => {
                            channel_id
                                .say(&http, format!("{}", state.get_dealer()))
                                .await
                                .unwrap();
                        }
                        Effect::Burst(name) => {
                            channel_id
                                .say(&http, format!("{name}さんはバーストしました。"))
                                .await
                                .unwrap();
                        }
                        Effect::DealerBurst => {
                            channel_id
                                .say(&http, "ディーラーがバーストしました。")
                                .await
                                .unwrap();
                        }
                        Effect::NextPlayer => match state.get_current_player() {
                            Some(player) => {
                                channel_id
                                    .say(
                                        &http,
                                        format!(
                                            "{}さんのターンです。コマンドを入力してください。",
                                            player.name
                                        ),
                                    )
                                    .await
                                    .unwrap();
                            }
                            None => {
                                channel_id
                                    .say(&http, "ディーラーのターンです")
                                    .await
                                    .unwrap();
                            }
                        },
                        Effect::NoPlayer => {
                            channel_id
                                .say(&http, "プレイヤーがいません。ゲームを終了します。")
                                .await
                                .unwrap();
                            break;
                        }
                        Effect::Finish => {
                            channel_id.say(&http, "ゲームが終了しました").await.unwrap();
                            channel_id.say(&http, state.to_string()).await.unwrap();
                            channel_id.say(&http, "結果を表示します").await.unwrap();
                            let result = state.get_result();
                            for (name, amount) in result.iter() {
                                save_result(&conn, UserId::Name(name.clone()), amount.0 as i32)
                                    .await
                                    .unwrap();
                            }
                            channel_id
                                .say(
                                    &http,
                                    result
                                        .iter()
                                        .map(|(name, amount)| {
                                            format!("{}: {} ({})", name, amount.0, amount.1)
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n"),
                                )
                                .await
                                .unwrap();
                        }
                    }
                }

                remove_table_tx.send(table_id).await.unwrap();
            });
        }

        Ok(CreateInteractionResponseMessage::new()
            .content("このチャンネルにゲームを登録しました".to_string()))
    }

    async fn register_user(
        &self,
        user_id: u64,
        name: String,
    ) -> Result<CreateInteractionResponseMessage, String> {
        create_discord_user(&self.conn, user_id, name.clone())
            .await
            .map_err(|_| "登録に失敗しました".to_string())?;

        Ok(CreateInteractionResponseMessage::new().content(format!("{} さんを登録しました", name)))
    }

    async fn get_balance(&self, user_id: u64) -> Result<CreateInteractionResponseMessage, String> {
        let balance = get_balance(&self.conn, UserId::Discord(user_id))
            .await
            .map_err(|_| "残高の取得に失敗しました".to_string())?;

        Ok(CreateInteractionResponseMessage::new()
            .content(format!("残高: {}", balance))
            .ephemeral(true))
    }

    async fn ping(
        &self,
        channel_id: u64,
        user_id: u64,
    ) -> Result<CreateInteractionResponseMessage, String> {
        let table_id = get_table_id(&self.conn, channel_id)
            .await
            .map_err(|_| "テーブルIDの取得に失敗しました".to_string())?;

        let game_tx = self
            .game_txs
            .lock()
            .unwrap()
            .get(&table_id)
            .ok_or("このチャンネルにはゲームが登録されていません".to_string())?
            .clone();

        let name = get_username_by_discord(&self.conn, user_id)
            .await
            .map_err(|_| "ユーザーが見つかりませんでした。登録してください。".to_string())?;

        let content = exec_game_command(game_tx, Command::Ping(name)).await;

        Ok(CreateInteractionResponseMessage::new().content(content))
    }

    async fn participate(
        &self,
        channel_id: u64,
        user_id: u64,
    ) -> Result<CreateInteractionResponseMessage, String> {
        let table_id = get_table_id(&self.conn, channel_id)
            .await
            .map_err(|_| "テーブルIDの取得に失敗しました".to_string())?;

        let game_tx = self
            .game_txs
            .lock()
            .unwrap()
            .get(&table_id)
            .ok_or("このチャンネルにはゲームが登録されていません".to_string())?
            .clone();

        let name = get_username_by_discord(&self.conn, user_id)
            .await
            .map_err(|_| "ユーザーが見つかりませんでした。登録してください。".to_string())?;

        let content = exec_game_command(game_tx, Command::Participate(name)).await;

        Ok(CreateInteractionResponseMessage::new().content(content))
    }

    async fn leave(
        &self,
        channel_id: u64,
        user_id: u64,
    ) -> Result<CreateInteractionResponseMessage, String> {
        let table_id = get_table_id(&self.conn, channel_id)
            .await
            .map_err(|_| "テーブルIDの取得に失敗しました".to_string())?;

        let game_tx = self
            .game_txs
            .lock()
            .unwrap()
            .get(&table_id)
            .ok_or("このチャンネルにはゲームが登録されていません".to_string())?
            .clone();

        let name = get_username_by_discord(&self.conn, user_id)
            .await
            .map_err(|_| "ユーザーが見つかりませんでした。登録してください。".to_string())?;

        let content = exec_game_command(game_tx, Command::Leave(name)).await;

        Ok(CreateInteractionResponseMessage::new().content(content))
    }

    async fn bet(
        &self,
        channel_id: u64,
        user_id: u64,
        amount: i32,
    ) -> Result<CreateInteractionResponseMessage, String> {
        let table_id = get_table_id(&self.conn, channel_id)
            .await
            .map_err(|_| "テーブルIDの取得に失敗しました".to_string())?;

        let game_tx = self
            .game_txs
            .lock()
            .unwrap()
            .get(&table_id)
            .ok_or("このチャンネルにはゲームが登録されていません".to_string())?
            .clone();

        let name = get_username_by_discord(&self.conn, user_id)
            .await
            .map_err(|_| "ユーザーが見つかりませんでした。登録してください。".to_string())?;

        let content = exec_game_command(game_tx, Command::Bet(name, amount as u32)).await;

        Ok(CreateInteractionResponseMessage::new().content(content))
    }

    async fn hit(
        &self,
        channel_id: u64,
        user_id: u64,
    ) -> Result<CreateInteractionResponseMessage, String> {
        let table_id = get_table_id(&self.conn, channel_id)
            .await
            .map_err(|_| "テーブルIDの取得に失敗しました".to_string())?;

        let game_tx = self
            .game_txs
            .lock()
            .unwrap()
            .get(&table_id)
            .ok_or("このチャンネルにはゲームが登録されていません".to_string())?
            .clone();

        let name = get_username_by_discord(&self.conn, user_id)
            .await
            .map_err(|_| "ユーザーが見つかりませんでした。登録してください。".to_string())?;

        let content = exec_game_command(game_tx, Command::Hit(name)).await;

        Ok(CreateInteractionResponseMessage::new().content(content))
    }

    async fn stand(
        &self,
        channel_id: u64,
        user_id: u64,
    ) -> Result<CreateInteractionResponseMessage, String> {
        let table_id = get_table_id(&self.conn, channel_id)
            .await
            .map_err(|_| "テーブルIDの取得に失敗しました".to_string())?;

        let game_tx = self
            .game_txs
            .lock()
            .unwrap()
            .get(&table_id)
            .ok_or("このチャンネルにはゲームが登録されていません".to_string())?
            .clone();

        let name = get_username_by_discord(&self.conn, user_id)
            .await
            .map_err(|_| "ユーザーが見つかりませんでした。登録してください。".to_string())?;

        let content = exec_game_command(game_tx, Command::Stand(name)).await;

        Ok(CreateInteractionResponseMessage::new().content(content))
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let user_id = command.user.id.get();
            let channel_id = command.channel_id.get();

            let result = match command.data.name.as_str() {
                "start" => self.start(ctx.http.clone(), command.channel_id).await,
                "ping" => self.ping(channel_id, user_id).await,
                "register" => {
                    let name = &command.data.options.first().unwrap().value;
                    let name = name.as_str().unwrap().to_string();
                    self.register_user(user_id, name).await
                }
                "balance" => self.get_balance(user_id).await,
                "participate" => self.participate(channel_id, user_id).await,
                "leave" => self.leave(channel_id, user_id).await,
                "bet" => {
                    let amount = &command.data.options.first().unwrap().value;
                    let amount = amount.as_i64().unwrap();
                    self.bet(channel_id, user_id, amount as i32).await
                }
                "hit" => self.hit(channel_id, user_id).await,
                "stand" => self.stand(channel_id, user_id).await,
                _ => Err("未知のコマンド".to_string()),
            };

            let data = match result {
                Ok(content) => content,
                Err(content) => CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true),
            };

            let builder = CreateInteractionResponse::Message(data);
            if let Err(why) = command.create_response(&ctx.http, builder).await {
                println!("Cannot respond to slash command: {why}");
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        for guild in ready.guilds {
            let guild_id = guild.id;

            let commands = guild_id
                .set_commands(
                    &ctx.http,
                    vec![
                        CreateCommand::new("start").description("チャンネルを登録"),
                        CreateCommand::new("ping").description("テスト用"),
                        CreateCommand::new("register")
                            .description("登録")
                            .add_option(
                                CreateCommandOption::new(CommandOptionType::String, "name", "名前")
                                    .required(true),
                            ),
                        CreateCommand::new("balance").description("残高"),
                        CreateCommand::new("participate").description("参加"),
                        CreateCommand::new("leave").description("退室"),
                        CreateCommand::new("bet").description("ベット").add_option(
                            CreateCommandOption::new(CommandOptionType::Integer, "amount", "金額")
                                .required(true),
                        ),
                        CreateCommand::new("hit").description("ヒット"),
                        CreateCommand::new("stand").description("スタンド"),
                    ],
                )
                .await;

            if let Err(why) = commands {
                println!("Cannot create slash commands: {why}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let conn = database::establish_connection().await.unwrap();
    let (remove_table_tx, mut remove_table_rx) = mpsc::channel(1);
    let handler = Arc::new(Handler {
        game_txs: Arc::new(Mutex::new(HashMap::new())),
        broadcast_txs: Arc::new(Mutex::new(HashMap::new())),
        remove_table_tx: remove_table_tx.clone(),
        conn,
    });

    {
        let handler = handler.clone();
        tokio::spawn(async move {
            while let Some(table_id) = remove_table_rx.recv().await {
                handler.game_txs.lock().unwrap().remove(&table_id);
                handler.broadcast_txs.lock().unwrap().remove(&table_id);
                delete_table(&handler.conn, table_id).await.unwrap();
            }
        });
    }

    let mut client = serenity::Client::builder(token, GatewayIntents::empty())
        .event_handler_arc(handler.clone())
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why}");
    }
}
