use std::collections::HashMap;

use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use serenity::model::guild::{Member};
use serenity::model::id::{GuildId, UserId, ChannelId, RoleId};

#[derive(Serialize, Deserialize, Debug)]
struct RaidInfo {
    raid_started: DateTime<Utc>,
    raiders: Vec<UserId>,
}

struct MemberShell {
    member: Member,
    current_pressure: u32,
    _log: HashMap<DateTime<Utc>, String>,
}

impl From<Member> for MemberShell {
    fn from(member: Member) -> Self {
        let mut shell = MemberShell { member, current_pressure: 0, _log: Default::default() };
        shell.log("Shell created!");
        shell
    }
}

impl MemberShell {
    fn log<S: AsRef<str>>(&mut self, content: S) {
        self._log.insert(Utc::now(), content.as_ref().parse().unwrap());
    }
}

#[derive(Serialize, Deserialize, Debug)]  // Serializing and deserializing channels will probably have to be reduced to their IDs, not the whole structs
pub struct GuildConfig {
    guild_id: GuildId,
    moderation_channel: Option<ChannelId>,
    raid_containment_channel: Option<ChannelId>,
    silence_containment_channel: Option<ChannelId>,
    log_channel: Option<ChannelId>,

    member_role: Option<RoleId>,
    silence_role: Option<RoleId>,
    new_role: Option<RoleId>,

    raid_trigger_timespan: u32,
    // Raid is triggered if n users join within this timespan, in seconds
    raid_trigger_new_user_limit: u32,
    raid_autoexpiration: u32,

    // Antispam pressure section
    max_pressure: f64,
    message_pressure: f64,
    embed_pressure: f64,
    character_pressure: f64,
    newline_pressure: f64,
    unique_ping_pressure: f64,
    pressure_decay_per_second: f64,
    // consider adding custom regex filters for pressure, as well as extra pressure for repeated messages
}

impl GuildConfig {
    fn new(guild_id: GuildId) -> Self {
        GuildConfig {
            guild_id,
            moderation_channel: None,
            raid_containment_channel: None,
            silence_containment_channel: None,
            log_channel: None,
            member_role: None,
            silence_role: None,
            new_role: None,
            raid_trigger_timespan: 90,
            raid_trigger_new_user_limit: 5,
            raid_autoexpiration: 600,
            max_pressure: 60.,
            message_pressure: 10.,
            embed_pressure: 8.3,
            character_pressure: 0.00625,
            newline_pressure: 0.714,
            unique_ping_pressure: 2.5,
            pressure_decay_per_second: 8.,
        }
    }
}


pub(crate) struct GuildShell {
    config: GuildConfig,
    current_raid: Option<RaidInfo>,
    last_raid: Option<RaidInfo>,
    active_members: HashMap<UserId, MemberShell>,
}

impl From<GuildConfig> for GuildShell {
    fn from(config: GuildConfig) -> Self {
        GuildShell {
            config,
            current_raid: None,
            last_raid: None,
            active_members: Default::default(),
        }
    }
}

impl Serialize for GuildShell {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.config.serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for GuildShell {
    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
        let config = GuildConfig::deserialize(deserializer)?;
        Ok(
            GuildShell {
                config,
                current_raid: None,
                last_raid: None,
                active_members: Default::default(),
            }
        )
    }
}

impl GuildShell {
    pub fn get_config(&self) -> &GuildConfig {
        &self.config
    }

    pub async fn member_joined(&mut self, ctx: &serenity::client::Context, new_member: Member) {
        let new_member_id = new_member.user.id.clone();
        let mut _shell = MemberShell::from(new_member);
        self.active_members.insert(new_member_id, _shell);
        let shell: &mut MemberShell = self.active_members.get_mut(&new_member_id).expect("Unreachable, we just inserted this member");

        if let Some(raid) = &mut self.current_raid {
            raid.raiders.push(new_member_id);
            shell.log("Joined during raid! No automatic role assignment");
        } else {
            if let Some(member_role) = &self.config.member_role {
                match shell.member.add_role(ctx, member_role).await {
                    Ok(_) => shell.log("Member role added"),
                    Err(e) => shell.log(format!("Adding member role failed! reason: {}", e))
                }
            } else { shell.log("Member role not configured so not assigned.") }
            if let Some(new_role) = &self.config.new_role {
                match shell.member.add_role(ctx, new_role).await {
                    Ok(_) => shell.log("'New' role added"),
                    Err(e) => shell.log(format!("Adding 'new' role failed! reason: {}", e))
                }
            } else { shell.log("'New'' role not configured so not assigned.") }
        }
    }
}
