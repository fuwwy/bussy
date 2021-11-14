mod guild_shell;
use guild_shell::*;

use std::env;

use dotenv::dotenv;
use serenity::{
    async_trait,
    framework::standard::macros::command,
    framework::standard::macros::group,
    model::{
        gateway::Ready,
        interactions::{
            application_command::ApplicationCommand,
            Interaction,
            InteractionResponseType,
        },
    },
    prelude::*,
};
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::framework::standard::CommandResult;
use serenity::framework::StandardFramework;
use serenity::model::id::ChannelId;
use std::io::{Read, Write};
use std::fmt::{Display, Formatter};

struct Handler;

struct BaseConfigData {
    debug_channel_id: ChannelId,
    bot_token: String,
    application_id: u64,
    shell_config_file: String
}

impl TypeMapKey for BaseConfigData {
    type Value = BaseConfigData;
}

struct GuildShells {}

impl TypeMapKey for GuildShells {
    type Value =Vec<GuildShell>;
}

#[group]
#[commands(ping)]
struct General;


#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let commands;
        if false {
            commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
                commands
                    .create_application_command(|command| {
                        command.name("ping").description("A ping command")
                    })
            })
                .await;
        } else {
            commands = ApplicationCommand::get_global_application_commands(&ctx.http).await;
            println!("Commands were not updated")
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        println!("INTERACTION RECEIVED");
        if let Interaction::ApplicationCommand(ref command) = interaction {
            // let author = command.member.expect("Command author");
            // let guild = ctx.cache().expect("Cache not present").guild(author.guild_id).await.expect("Guild not retrievable");


            let content = match command.data.name.as_str() {
                "ping" => {
                    "Pong!"
                }
                _ => { "empty" }
            };

            if content != "" {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| { message.content(content) })
                })
                    .await;
            };
        }
    }
}


fn load_shells(shell_file: &str) -> Vec<GuildShell> {

    let mut file = std::fs::File::open(shell_file).expect("File could not be opened");

    let mut data = String::new();
    file.read_to_string(&mut data);
    let deserialized: Vec<GuildConfig> =  serde_yaml::from_str(&*data).expect("File could not be deserialized");
    let shells = deserialized.into_iter().map(|f| GuildShell::deserialized(f)).collect();

    save_shells(&shells, shell_file);  // Instant check whether the file can also be saved.
    shells
}


fn save_shells(shells: &Vec<GuildShell>, shell_file: &str) {
    let mut file = std::fs::File::open(shell_file).expect("Can't open file ");
    let shell_configs: Vec<&GuildConfig> = shells.into_iter().map(|f| f.get_config()).collect();
    let serialized = serde_yaml::to_string(&shell_configs).expect("Can't serialize shells");
    file.write(serialized.as_ref());
}

async fn run(config: BaseConfigData) {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Build our client.
    let mut client = Client::builder(&config.bot_token)
        .event_handler(Handler)
        .framework(framework)
        .application_id(config.application_id)
        .intents(GatewayIntents::all())
        .await
        .expect("Error creating client");

    {
        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;
        data.insert::<GuildShells>(load_shells(&*config.shell_config_file));
        data.insert::<BaseConfigData>(config);
    }


    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
async fn ping() -> CommandResult {
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("BOT_TOKEN").expect("Bot token in .env");

    // The Application Id is usually the Bot User Id.
    let application_id: u64 = env::var("APPLICATION_ID").expect("Application ID in .env").parse::<u64>().expect("Application Id to be a u64");


    let debug_channel_id = env::var("DEBUG_CHANNEL_ID")
        .expect("DEBUG_CHANNEL_ID in environment").parse::<ChannelId>()
        .expect("Debug channel ID to be a valid channel id");

    let config = BaseConfigData { debug_channel_id: debug_channel_id, bot_token: token.to_string(), application_id, shell_config_file: "shells.yml".to_string() };

    run(config).await;
}
