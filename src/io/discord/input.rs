// It is recommended that you read the README file, it is very important to this example.
// This example will help us to use a sqlite database with our bot.
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::env;
use tokio::sync::mpsc::Sender;

use serenity::async_trait;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::{
    db::user_id::UserId,
    utils::command::{Command, CommandData},
};

pub struct Handler {
    pub(crate) tx: Mutex<Sender<Command>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let user_id = command.user.id.get() as i32;
            let user_id = UserId::Discord(user_id);

            let data = match command.data.name.as_str() {
                "register" => {
                    println!("{:?}", command.data.options);
                    let name = &command.data.options.first().unwrap().value;
                    let name = name.as_str().unwrap();
                    CommandData::Register(name.to_string())
                }
                "rename" => {
                    let new_name = &command.data.options.first().unwrap().value;
                    let new_name = new_name.as_str().unwrap();
                    CommandData::Rename(new_name.to_string())
                }
                "balance" => CommandData::Balance,
                "get_bonus" => {
                    let bonus_id = &command.data.options.first().unwrap().value;
                    let bonus_id = bonus_id.as_i64().unwrap();
                    CommandData::GetBonus(bonus_id as i32)
                }
                "participate" => CommandData::Participate,
                "leave" => CommandData::Leave,
                "bet" => {
                    let amount = &command.data.options.first().unwrap().value;
                    let amount = amount.as_i64().unwrap();
                    CommandData::Bet(amount as u32)
                }
                "hit" => CommandData::Hit,
                "stand" => CommandData::Stand,
                _ => return,
            };
            println!("Received command: {:?}", command.data);

            let input_command = Command::new(data, user_id);
            self.tx.lock().await.send(input_command).await.unwrap();

            let data = CreateInteractionResponseMessage::new().content("ok");
            let builder = CreateInteractionResponse::Message(data);
            if let Err(why) = command.create_response(&ctx.http, builder).await {
                println!("Cannot respond to slash command: {why}");
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = guild_id
            .set_commands(
                &ctx.http,
                vec![
                    CreateCommand::new("register")
                        .description("登録")
                        .add_option(
                            CreateCommandOption::new(CommandOptionType::String, "name", "名前")
                                .required(true),
                        ),
                    CreateCommand::new("rename")
                        .description("名前変更")
                        .add_option(
                            CreateCommandOption::new(
                                CommandOptionType::String,
                                "new_name",
                                "新しい名前",
                            )
                            .required(true),
                        ),
                    CreateCommand::new("balance").description("残高確認"),
                    CreateCommand::new("get_bonus")
                        .description("ボーナス取得")
                        .add_option(
                            CreateCommandOption::new(
                                CommandOptionType::Integer,
                                "bonus_id",
                                "ボーナスID",
                            )
                            .required(true),
                        ),
                    CreateCommand::new("participate").description("参加"),
                    CreateCommand::new("leave").description("退出"),
                    CreateCommand::new("bet").description("ベット").add_option(
                        CreateCommandOption::new(CommandOptionType::Integer, "amount", "金額")
                            .required(true),
                    ),
                    CreateCommand::new("hit").description("ヒット"),
                    CreateCommand::new("stand").description("スタンド"),
                ],
            )
            .await;

        // println!("I now have the following guild slash commands: {commands:#?}");
    }
}

pub async fn run(client: &mut Client) -> Result<(), serenity::Error> {
    client.start().await
}
