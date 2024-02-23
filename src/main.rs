use std::env;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use database::{
    create_discord_user, create_table, get_balance, get_table_id, get_username_by_discord, UserId,
};
use dotenvy::dotenv;
use serenity::all::ChannelId;
use serenity::async_trait;
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::client::{Context, EventHandler};
use serenity::model::prelude::{CommandOptionType, GatewayIntents, Interaction, Ready};
use sqlx::{Pool, Postgres};

mod database;

pub enum Command {
    Ping(String),
}

#[derive(Debug)]
pub struct Response {
    content: String,
}

pub struct Request {
    res_tx: tokio::sync::oneshot::Sender<Response>,
    command: Command,
}

pub struct Handler {
    game_txs: Arc<Mutex<HashMap<i32, tokio::sync::mpsc::Sender<Request>>>>,
    broadcast_txs: Arc<Mutex<HashMap<i32, tokio::sync::broadcast::Sender<String>>>>,
    conn: Pool<Postgres>,
}

pub async fn exec_game_command(
    game_tx: tokio::sync::mpsc::Sender<Request>,
    command: Command,
) -> String {
    let (res_tx, res_rx) = tokio::sync::oneshot::channel::<Response>();

    let request = Request { res_tx, command };

    game_tx.send(request).await.unwrap();
    res_rx.await.unwrap().content
}

impl Handler {
    async fn register_channel(
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
        let (game_tx, mut game_rx) = tokio::sync::mpsc::channel(1);
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.game_txs.lock().unwrap().entry(table_id)
        {
            e.insert(game_tx);
        } else {
            return Err("このチャンネルには既にゲームが登録されています".to_string());
        };

        let (broadcast_tx, _) = tokio::sync::broadcast::channel(1);
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.broadcast_txs.lock().unwrap().entry(table_id)
        {
            e.insert(broadcast_tx.clone());
        } else {
            return Err("このチャンネルには既にゲームが登録されています".to_string());
        };
        {
            let broadcast_tx = broadcast_tx.clone();
            tokio::spawn(async move {
                loop {
                    let request = game_rx.recv().await.unwrap();
                    let content = match request.command {
                        Command::Ping(name) => format!("{} さん、こんにちは", name),
                    };
                    let response = Response { content };
                    request.res_tx.send(response).unwrap();

                    for i in 0..10 {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        broadcast_tx.send(format!("test message: {}", i)).unwrap();
                    }
                }
            });
        }

        tokio::spawn(async move {
            let mut broadcast_rx = broadcast_tx.subscribe();
            loop {
                let message = broadcast_rx.recv().await.unwrap();
                channel_id
                    .say(&http, format!("broadcast: {}", message))
                    .await
                    .unwrap();
            }
        });

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
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let user_id = command.user.id.get();
            let channel_id = command.channel_id.get();

            let result = match command.data.name.as_str() {
                "register_channel" => {
                    self.register_channel(ctx.http.clone(), command.channel_id)
                        .await
                }
                "ping" => self.ping(channel_id, user_id).await,
                "register" => {
                    let name = &command.data.options.first().unwrap().value;
                    let name = name.as_str().unwrap().to_string();
                    self.register_user(user_id, name).await
                }
                "balance" => self.get_balance(user_id).await,
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
                        CreateCommand::new("register_channel").description("チャンネルを登録"),
                        CreateCommand::new("ping").description("テスト用"),
                        CreateCommand::new("register")
                            .description("登録")
                            .add_option(
                                CreateCommandOption::new(CommandOptionType::String, "name", "名前")
                                    .required(true),
                            ),
                        CreateCommand::new("balance").description("残高"),
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
    let handler = Arc::new(Handler {
        game_txs: Arc::new(Mutex::new(HashMap::new())),
        broadcast_txs: Arc::new(Mutex::new(HashMap::new())),
        conn,
    });

    let mut client = serenity::Client::builder(token, GatewayIntents::empty())
        .event_handler_arc(handler.clone())
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why}");
    }
}
