use std::env;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use database::{create_discord_user, get_balance, get_username_by_discord, UserId};
use dotenvy::dotenv;
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
    game_txs: Arc<Mutex<HashMap<u64, tokio::sync::mpsc::Sender<Request>>>>,
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

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let user_id = command.user.id.get();
            eprintln!("user_id: {}", user_id);
            let channel_id = command.channel_id.get();

            let data = match command.data.name.as_str() {
                "new" => {
                    let (game_tx, mut game_rx) = tokio::sync::mpsc::channel(1);
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        self.game_txs.lock().unwrap().entry(channel_id)
                    {
                        e.insert(game_tx);

                        tokio::spawn(async move {
                            loop {
                                let request = game_rx.recv().await.unwrap();
                                let content = match request.command {
                                    Command::Ping(name) => format!("{} さん、こんにちは", name),
                                };
                                let response = Response { content };
                                request.res_tx.send(response).unwrap();
                            }
                        });
                        let content = "このチャンネルにゲームを登録しました".to_string();
                        CreateInteractionResponseMessage::new().content(content)
                    } else {
                        let content = "このチャンネルには既にゲームが登録されています".to_string();
                        CreateInteractionResponseMessage::new()
                            .content(content)
                            .ephemeral(true)
                    }
                }
                "ping" => {
                    let game_tx = match self.game_txs.lock().unwrap().get(&channel_id) {
                        Some(game_tx) => Ok(game_tx.clone()),
                        None => Err("このチャンネルにはゲームが登録されていません".to_string()),
                    };
                    match game_tx {
                        Ok(game_tx) => match get_username_by_discord(&self.conn, user_id).await {
                            Ok(name) => {
                                let content = exec_game_command(game_tx, Command::Ping(name)).await;
                                CreateInteractionResponseMessage::new().content(content)
                            }
                            Err(content) => CreateInteractionResponseMessage::new()
                                .content(content.to_string())
                                .ephemeral(true),
                        },
                        Err(content) => CreateInteractionResponseMessage::new()
                            .content(content)
                            .ephemeral(true),
                    }
                }
                "register" => {
                    let name = &command.data.options.first().unwrap().value;
                    let name = name.as_str().unwrap().to_string();

                    let content = match create_discord_user(&self.conn, user_id, name.clone()).await
                    {
                        Ok(_) => {
                            format!("登録しました: {}", name)
                        }
                        Err(_) => "登録に失敗しました".to_string(),
                    };

                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .ephemeral(true)
                }
                "balance" => {
                    let content = match get_balance(&self.conn, UserId::Discord(user_id)).await {
                        Ok(balance) => format!("残高: {}", balance),
                        Err(_) => "残高の取得に失敗しました".to_string(),
                    };
                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .ephemeral(true)
                }
                _ => CreateInteractionResponseMessage::new()
                    .content("未知のコマンド".to_string())
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
                        CreateCommand::new("new").description("チャンネルを登録"),
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
    let handler = Handler {
        game_txs: Arc::new(Mutex::new(HashMap::new())),
        conn,
    };

    let mut client = serenity::Client::builder(token, GatewayIntents::empty())
        .event_handler(handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why}");
    }
}
