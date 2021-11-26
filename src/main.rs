use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};

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
use serenity::model::guild::{Member, Guild};
use serenity::model::id::{ChannelId, GuildId};


use guild_shell::*;
use serenity::model::channel::{Message};




use serenity::model::interactions::application_command::ApplicationCommandOptionType;


mod guild_shell;
mod config_form;

struct Handler;

struct BaseConfigData {
    debug_channel_id: ChannelId,
    bot_token: String,
    application_id: u64,
    shell_config_file: String,
}

impl TypeMapKey for BaseConfigData {
    type Value = BaseConfigData;
}

struct GuildShells {}

impl TypeMapKey for GuildShells {
    type Value = HashMap<GuildId, GuildShell>;
}

#[group]
#[commands(ping, recreate_shell)]
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
                    .create_application_command(|cmd| {
                        cmd.name("config").description("Configure the server settings for bussy")
                    })
                    .create_application_command(|cmd| {
                        cmd.name("reset_guild_shell").description("Resets the guild settings to default values")
                    })
                    .create_application_command(|cmd| {
                        cmd.name("load_settings").description("Load settings from a raw JSON data")
                            .create_option(|opt| {
                                opt.name("settings").description("Settings to load").kind(ApplicationCommandOptionType::String)
                            })
                    })
                    .create_application_command(|cmd| {
                        cmd.name("dump_settings").description("Dumps the current descriptions as a JSON file")
                    })
            })
                .await;
        } else {
            commands = ApplicationCommand::get_global_application_commands(&ctx.http).await;
            println!("Commands were not updated")
        }
        println!("There is {} commands registered", commands.expect("Commands failed to retrieve.").len());

        // Ensure a shell for all guilds
        let guilds = ctx.cache.guilds().await;
        let mut data = ctx.data.write().await;
        let shell_filename = &*data.get::<BaseConfigData>().unwrap().shell_config_file.clone();
        let shells = data.get_mut::<GuildShells>().unwrap();
        for id in guilds {
            if !shells.contains_key(&id) {
                shells.insert(id, GuildShell::from(id));
                println!("Shell created for guild {} on start", id);
            }
        }

        save_shells(&shells, shell_filename);
    }

    async fn guild_member_addition(&self, _ctx: Context, _guild_id: GuildId, _new_member: Member) {
        let mut data = _ctx.data.write().await;
        if let Some(target_guild) = data.get_mut::<GuildShells>().expect("Guild shells must be present in client data").get_mut(&_guild_id) {
            target_guild.member_joined(&_ctx, _new_member).await;
        } else {
            println!("AAA guild shell not set up for guild id {}", _guild_id);  // TODO: implement better error handling
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        println!("INTERACTION RECEIVED");
        if let Interaction::ApplicationCommand(ref command) = interaction {
            // let author = command.member.expect("Command author");
            // let guild = ctx.cache().expect("Cache not present").guild(author.guild_id).await.expect("Guild not retrievable");


            let content: String = match command.data.name.as_str() {
                "ping" => {
                    "Pong!".into()
                }
                "reset_guild_shell" => {
                    match _recreate_shell(&ctx, &command.guild_id.unwrap_or(910640456457666631.into())).await {
                        Ok(_) => "cool".into(),
                        Err(e) => format!("Not cool :( {}", e)
                    }

                }
                "load_settings" => {
                    let settings: &str = command.data.options.get(0).expect("Settings must be present").value.as_ref().expect("Must have value").as_str().expect("Must be string");
                    let guild_id = command.guild_id.unwrap_or(GuildId::from(910640456457666631));
                    match serde_json::from_str(settings) {
                        Ok(config) => {
                            let mut data = ctx.data.write().await;
                            let shells = data.get_mut::<GuildShells>().unwrap();
                            shells.get_mut(&guild_id).expect("guild shell must exist").config = config;
                            save_shells(data.get::<GuildShells>().unwrap(), &*data.get::<BaseConfigData>().unwrap().shell_config_file);
                            println!("loaded successfully!!")
                        }
                        Err(e) => println!("Couldnt parse :( {}", e)
                    }
                    "done (maybe, idk)".into()

                }
                "dump_settings" => {
                    let data = ctx.data.read().await;
                    let guild_id = command.guild_id.unwrap_or(GuildId::from(910640456457666631));
                    let shell = data.get::<GuildShells>().unwrap().get(&guild_id).expect("Guild shell must exist");

                    match serde_json::to_string(shell) {
                        Ok(res) => format!("```json\n{}```", res),
                        Err(e) => format!("```Failed to convert config into JSON! This should never happen. Error: {}```", e)
                    }
                }
                "config" => {
                    let mut data = ctx.data.write().await;
                    let guild_id = command.guild_id.unwrap_or(GuildId::from(910640456457666631));
                    let mut shell = data.get_mut::<GuildShells>().unwrap().get_mut(&guild_id).expect("Guild shell must exist");

                    if let Err(e) = shell.handle_interaction(&ctx, &interaction).await {
                        e.to_string()
                    } else {
                        "".into()
                    }
                }
                _ => { "empty".into() }
            };

            if content != "" {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| { message.content(content) })
                })
                    .await.expect("Failed to create interaction response");
            };
        } else {
            let mut data = ctx.data.write().await;
            for mut shell in data.get_mut::<GuildShells>().unwrap().values_mut() {
                shell.handle_interaction(&ctx, &interaction);
            }
        }
    }
    async fn guild_create(&self, _ctx: Context, _guild: Guild, _is_new: bool) {
        println!("New guild! Id: {}, is new: {}", _guild.id, _is_new);
        let mut data = _ctx.data.write().await;
        let shells = data.get_mut::<GuildShells>().unwrap();
        if !shells.contains_key(&_guild.id) {
            shells.insert(_guild.id, GuildShell::from(_guild.id));
        }
    }
    async fn message(&self, ctx: Context, msg: Message) {
        let mut data = ctx.data.write().await;
        if let Some(guild_id) = &msg.guild_id {
            if let Some(shell) = data.get_mut::<GuildShells>().unwrap().get_mut(guild_id) {
                shell.message_created(&ctx, &msg).await;
            }
            else {
                println!("Guild {} has no shell", guild_id)
            }
        } // Else is a DM
        println!("MESSAGE received aaaa");
    }
}


fn load_shells(shell_file: &str) -> HashMap<GuildId, GuildShell> {
    let mut file = std::fs::File::open(shell_file).expect("File could not be opened");

    let mut data = String::new();
    file.read_to_string(&mut data).expect("File couldn't be read into string");
    let deserialized: HashMap<GuildId, GuildConfig> = serde_yaml::from_str(&*data).expect("File could not be deserialized");
    let mut shells: HashMap<GuildId, GuildShell> = Default::default();
    for (id, config) in deserialized {
        shells.insert(id, GuildShell::from(config));
    }

    save_shells(&shells, shell_file);  // Instant check whether the file can also be saved.
    shells
}


fn save_shells(shells: &HashMap<GuildId, GuildShell>, shell_file: &str) {
    let mut file = std::fs::File::create(shell_file).expect("Can't open file ");
    let mut shell_configs: HashMap<GuildId, &GuildConfig> = Default::default();

    for (id, shell) in shells {
        shell_configs.insert(id.clone(), shell.get_config());
    }

    let serialized = serde_yaml::to_string(&shell_configs).expect("Can't serialize shells");
    file.write(serialized.as_ref()).expect("File couldn't be written into");
}

async fn run(config: BaseConfigData) {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::default();
    intents.guild_members();
    intents.direct_messages();

    // Build our client.
    let mut client = Client::builder(&config.bot_token)
        .event_handler(Handler)
        .framework(framework)
        .application_id(config.application_id)
        .intents(intents)
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
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Pinged!");
    let channel = msg.channel(ctx).await.unwrap();
    if let serenity::model::channel::Channel::Guild(ch) = channel {
        ch.say(&ctx, "Pong").await.expect("Couldn't respond to ping!");
    }
    Ok(())
}

#[command]
async fn recreate_shell(ctx: &Context, msg: &Message) -> CommandResult {
    _recreate_shell(&ctx, &msg.guild_id.expect("Guild id")).await
}

async fn _recreate_shell(ctx: &Context, guild_id: &GuildId) -> CommandResult{
    println!("Got recreate shell command");
    let mut data = ctx.data.write().await;
    let config_file = &data.get::<BaseConfigData>().unwrap().shell_config_file.clone();
    let shells = data.get_mut::<GuildShells>().unwrap();
    shells.remove(guild_id);
    let shell = GuildShell::from(guild_id.clone());
    shells.insert(guild_id.clone(), shell);

    save_shells(shells, config_file);
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
