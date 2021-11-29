use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};
use std::sync::Arc;

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
use serenity::model::channel::Message;
use serenity::model::guild::{Guild, Member};
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::interactions::application_command::ApplicationCommandOptionType;
use tokio::sync::*;
use tokio::task::JoinHandle;

use guild_shell::*;

mod guild_shell;
mod config_form;
mod error_handling;

struct ShellContact {
    channel: mpsc::Sender<ShellEvent>,
    handle: JoinHandle<()>,
}

type ShellMap = HashMap<GuildId, ShellContact>;

struct Handler;

enum ShellEvent {
    NewMessage(Context, Message),
    MemberJoined(Context, Member),
    NewInteraction(Context, Interaction),
    GetConfig(oneshot::Sender<GuildConfig>),
}

struct GuildShells {}

impl TypeMapKey for GuildShells {
    type Value = HashMap<GuildId, ShellContact>;
}

struct BaseConfigData {
    debug_channel_id: ChannelId,
    bot_token: String,
    application_id: u64,
    shell_config_file: String,
}

impl TypeMapKey for BaseConfigData {
    type Value = BaseConfigData;
}


#[group]
#[commands(ping, recreate_shell)]
struct General;


#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, mut ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let mut some_config = GuildConfig::new(0.into());

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
                    .create_application_command(|cmd| {
                        cmd.name("change").description("Change a setting manually");
                        for field in some_config.get_configurable_fields() {
                            field.add_slash_command_subcommand(cmd);
                        }
                        cmd
                    })
            })
                .await;
        } else {
            commands = ApplicationCommand::get_global_application_commands(&ctx.http).await;
            println!("Commands were not updated")
        }
        println!("There is {} commands registered", commands.expect("Commands failed to retrieve.").len());


        load_shells(&mut ctx).await;

        // Ensure a shell for all guilds
        let guilds = ctx.cache.guilds().await;
        let mut data = ctx.data.write().await;
        let shells = data.get_mut::<GuildShells>().unwrap();

        for id in guilds {
            if !shells.contains_key(&id) {
                GuildShell::initialize(&ctx, GuildConfig::new(id)).await;
                println!("Shell created for guild {} on start", id);
            }
        }
        drop(data);

        save_shells(&mut ctx.data).await;

        /*
        let loopctx = ctx.clone();
        let mut interval_timer = tokio::time::interval(chrono::Duration::seconds(5).to_std().unwrap());



        tokio::spawn(async move {
            loop {
                interval_timer.tick().await;
                // println!("Tick");
                let mut data = loopctx.data.write().await;
                let shells = data.get_mut::<GuildShells>().unwrap();
                for shell in shells.values_mut() {
                    if !shell.dump_logs(&loopctx.clone()).await.is_ok() {
                        println!("Couldn't dump logs for {}!", shell.config.guild_id);
                    }
                }
            }
        });

         */
    }

    async fn guild_member_addition(&self, ctx: Context, _guild_id: GuildId, new_member: Member) {
        let mut data = ctx.data.write().await;
        if let Some(target_guild) = data.get_mut::<GuildShells>().unwrap().get_mut(&_guild_id) {
            target_guild.channel.send(ShellEvent::MemberJoined(ctx.clone(), new_member)).await;
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
                    match _recreate_shell(&ctx, &command.guild_id.expect("Must be used in a guild")).await {
                        Ok(_) => "cool".into(),
                        Err(e) => format!("Not cool :( {}", e)
                    }
                }
                /*
                "load_settings" => {
                    let settings: &str = command.data.options.get(0).expect("Settings must be present").value.as_ref().expect("Must have value").as_str().expect("Must be string");
                    let guild_id = command.guild_id.expect("Command must be used in a guild");
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
                    let guild_id = command.guild_id.unwrap();
                    let shell = data.get::<GuildShells>().unwrap().get(&guild_id).expect("Guild shell must exist");

                    match serde_json::to_string(shell) {
                        Ok(res) => format!("```json\n{}```", res),
                        Err(e) => format!("```Failed to convert config into JSON! This should never happen. Error: {}```", e)
                    }
                }*/

                /*"config" => {
                    let mut data = ctx.data.write().await;
                    let guild_id = command.guild_id.unwrap_or(GuildId::from(910640456457666631));
                    let shell = data.get_mut::<GuildShells>().unwrap().get_mut(&guild_id).expect("Guild shell must exist");

                    if let Err(e) = shell.handle_interaction(&ctx, &interaction).await {
                        e.to_string()
                    } else {
                        "".into()
                    }
                }*/
                _ => { "".into() }
            };

            if content != "" {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| { message.content(content) })
                })
                    .await.expect("Failed to create interaction response");
            };
        }

        // All shells handle interaction
        let data = ctx.data.write().await;

        let guild_id = {
            match &interaction {
                Interaction::Ping(_ping) => { return; }
                Interaction::ApplicationCommand(cmd) => { cmd.guild_id }
                Interaction::MessageComponent(cmp) => { cmp.guild_id }
            }
        }.expect("Interaction must have guild id i guess");

        let shell = data.get::<GuildShells>().unwrap().get(&guild_id).expect("nonexistent guild smh");
        shell.channel.send(ShellEvent::NewInteraction(ctx.clone(), interaction)).await;
    }
    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: bool) {
        println!("New guild! Id: {}, is new: {}", guild.id, _is_new);
        GuildShell::initialize(&ctx, GuildConfig::new(guild.id)).await;
    }
    async fn message(&self, ctx: Context, msg: Message) {
        let mut data = ctx.data.write().await;
        println!("{} said {}...", msg.author.name, "something");
        if let Some(guild_id) = &msg.guild_id {
            if let Some(shell) = data.get_mut::<GuildShells>().unwrap().get_mut(guild_id) {
                shell.channel.send(ShellEvent::NewMessage(ctx.clone(), msg)).await;
            } else {
                println!("Guild {} has no shell", guild_id)
            }
        } // Else is a DM
    }
}


async fn load_shells(ctx: &mut Context) {
    let filename = ctx.data.read().await.get::<BaseConfigData>().unwrap().shell_config_file.clone();
    let mut file = std::fs::File::open(filename).expect("File could not be opened");

    let mut data = String::new();
    file.read_to_string(&mut data).expect("File couldn't be read into string");
    let deserialized: HashMap<GuildId, GuildConfig> = serde_yaml::from_str(&*data).expect("File could not be deserialized");
    for (_id, config) in deserialized {
        GuildShell::initialize(ctx, config).await;
    }

    save_shells(&mut ctx.data).await;  // Instant check whether the file can also be saved.
}


async fn save_shells(dat: &mut Arc<RwLock<TypeMap>>) {
    let mut data = dat.write().await;
    let filename = data.get::<BaseConfigData>().unwrap().shell_config_file.clone();
    let shells = data.get_mut::<GuildShells>().unwrap();

    let mut file = std::fs::File::create(filename).expect("Can't open file ");
    let mut shell_configs: HashMap<GuildId, GuildConfig> = Default::default();

    for (id, shell) in shells {  // TODO: Rewrite to collect configs concurrently
        let (sender, receiver) = oneshot::channel();
        shell.channel.send(ShellEvent::GetConfig(sender)).await;
        if let Ok(config) = receiver.await {
            shell_configs.insert(id.clone(), config);
        } else {
            println!("Error retrieving the config for guild {}", id);
        }
    }

    let serialized = serde_yaml::to_string(&shell_configs).expect("Can't serialize shells");
    file.write(serialized.as_ref()).expect("File couldn't be written into");
}

async fn run(config: BaseConfigData) {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::all();
    // intents.guild_members();
    // intents.direct_messages();


    // Build our client.
    let mut client = Client::builder(&config.bot_token)
        .event_handler(Handler)
        .framework(framework)
        .application_id(config.application_id)
        .intents(intents)
        .await
        .expect("Error creating client");


    {
        let mut data = client.data.write().await;
        data.insert::<GuildShells>(Default::default());
        data.insert::<BaseConfigData>(config);
        data.insert::<LogData>(LogData::default());
    }


    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });


    if let Err(why) = client.start().await {
        println!("Cleint exit with error: {}", why);
    }

    {
        save_shells(&mut client.data).await;
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

async fn _recreate_shell(_ctx: &Context, _guild_id: &GuildId) -> CommandResult {
    todo!();
    /*
    println!("Got recreate shell command");
    let mut data = ctx.data.write().await;
    let config_file = &data.get::<BaseConfigData>().unwrap().shell_config_file.clone();
    let shells = data.get_mut::<GuildShells>().unwrap();
    shells.remove(guild_id);

    GuildShell::initialize(ctx, GuildConfig::new(guild_id.clone())).await;

    save_shells(&mut ctx.data).await;
    Ok(())

     */
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
