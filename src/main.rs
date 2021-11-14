use std::env;

use dotenv::dotenv;
use serenity::{
    async_trait,
    framework::standard::macros::command,
    framework::standard::macros::group,
    model::{
        gateway::Ready,
        interactions::{
            application_command::{
                ApplicationCommand,
                ApplicationCommandOptionType,
            },
            Interaction,
            InteractionResponseType,
        },
    },
    prelude::*,
};
use serenity::client::bridge::gateway::event::ShardStageUpdateEvent;
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::framework::standard::CommandResult;
use serenity::framework::StandardFramework;
use serenity::futures::future::err;
use serenity::model::channel::{Channel, ChannelCategory, Embed, GuildChannel, Message, PartialGuildChannel, Reaction, StageInstance};
use serenity::model::event::{ChannelPinsUpdateEvent, GuildMembersChunkEvent, GuildMemberUpdateEvent, InviteCreateEvent, InviteDeleteEvent, MessageUpdateEvent, PresenceUpdateEvent, ResumedEvent, ThreadListSyncEvent, ThreadMembersUpdateEvent, TypingStartEvent, VoiceServerUpdateEvent};
use serenity::model::gateway::Presence;
use serenity::model::guild::{Emoji, Guild, GuildUnavailable, Integration, Member, PartialGuild, Role, ThreadMember};
use serenity::model::id::{ApplicationId, ChannelId, EmojiId, GuildId, IntegrationId, MessageId, RoleId};
use serenity::model::invite::RichInvite;
use serenity::model::prelude::{CurrentUser, User, VoiceState};
use serenity::utils::MessageBuilder;
use tokio::sync::mpsc::{channel, Receiver, Sender};

struct Handler;

struct BaseConfig;

struct BaseConfigData {
    debug_channel_id: ChannelId,
}

impl TypeMapKey for BaseConfig {
    type Value = BaseConfigData;
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

pub async fn run(token: String, application_id: u64) {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Build our client.
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .application_id(application_id)
        .intents(GatewayIntents::all())
        .await
        .expect("Error creating client");

    {
        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;


        let debug_channel_id = env::var("DEBUG_CHANNEL_ID")
            .expect("DEBUG_CHANNEL_ID in environment").parse::<ChannelId>()
            .expect("Debug channel ID to be a valid channel id");

        let config = BaseConfigData { debug_channel_id: debug_channel_id };

        data.insert::<BaseConfig>(config);
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

    run(token, application_id).await;
}
