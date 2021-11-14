use serenity::model::channel::GuildChannel;
use serenity::model::guild::{Role, Guild, Member};
use chrono::prelude::*;
use std::collections::HashMap;
use serenity::model::id::{UserId, GuildId};

use serde::{Serialize, Deserialize, Serializer};
use serde::ser::{Error};

#[derive(Serialize, Deserialize, Debug)]
struct RaidInfo {
    raid_started: DateTime<Utc>
}

struct MemberShell {
    member: Member,
    current_pressure: u32,
}


#[derive(Serialize, Deserialize, Debug)]  // Serializing and deserializing channels will probably have to be reduced to their IDs, not the whole structs
pub struct GuildConfig {
    guild_id: GuildId,
    moderation_channel: Option<GuildChannel>,
    raid_containment_channel: Option<GuildChannel>,
    silence_containment_channel: Option<GuildChannel>,
    log_channel: Option<GuildChannel>,

    member_role: Option<Role>,
    silence_role: Option<Role>,
    new_role: Option<Role>,

    raid_trigger_timespan: u32,  // Raid is triggered if n users join within this timespan, in seconds
    raid_trigger_new_user_limit: u32,
    raid_autoexpiration: u32,

    // Antispam pressure section
    max_pressure: u32,
    message_pressure: u32,
    embed_pressure: u32,
    character_pressure: u32,
    newline_pressure: u32,
    unique_ping_pressure: u32,
    pressure_decay_per_second: u32
    // consider adding custom regex filters for pressure, as well as extra pressure for repeated messages
}


pub(crate) struct GuildShell {
    config: GuildConfig,
    current_raid: Option<RaidInfo>,
    last_raid: Option<RaidInfo>,
    active_members: HashMap<UserId, MemberShell>
}

impl GuildShell {
    pub fn deserialized(config: GuildConfig) -> GuildShell{
        GuildShell {
            config,
            current_raid: None,
            last_raid: None,
            active_members: Default::default()
        }
    }

    pub fn get_config(&self) -> &GuildConfig {
        &self.config
    }
}
