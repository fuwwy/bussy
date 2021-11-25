use std::collections::HashMap;

use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serenity::model::guild::{Member};
use serenity::model::id::{GuildId, UserId, ChannelId, RoleId};
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::Error;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
struct RaidInfo {
    raid_started: DateTime<Utc>,
    raiders: Vec<UserId>,
}

struct MemberShell {
    member: Member,
    current_pressure: f64,
    _log: HashMap<DateTime<Utc>, String>,
    last_pressure_decay: chrono::DateTime<Utc>
}

impl From<Member> for MemberShell {
    fn from(member: Member) -> Self {
        let mut shell = MemberShell { member, current_pressure: 0., _log: Default::default(), last_pressure_decay: Utc::now() };
        shell.log("Shell created!");
        shell
    }
}

impl MemberShell {
    fn log<S: AsRef<str>>(&mut self, content: S) {
        self._log.insert(Utc::now(), content.as_ref().parse().unwrap());
    }

    fn update_pressure(&mut self, decay_per_second: &f64, add_pressure: &f64) -> f64 {
        let current_time = Utc::now();
        let to_decay: f64 = (current_time - self.last_pressure_decay).num_seconds() as f64 * decay_per_second;
        if to_decay > self.current_pressure {
            self.current_pressure = 0.;
        } else {
            self.current_pressure -= to_decay;
        }
        self.last_pressure_decay = current_time;
        self.current_pressure += add_pressure;
        self.current_pressure
    }
}

#[derive(Debug)]
struct ConfigField<T> {
    _inner: T,
    // name: String
}

impl<T> std::ops::Deref for ConfigField<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        return &self._inner
    }
}

impl<T> ConfigField<T> where
T: FromStr {
    pub fn set_value(&mut self, new_value: String) -> Result<(), <T as FromStr>::Err> {
        self._inner = new_value.parse()?;
        Ok(())
    }
}

impl<T> From<T> for ConfigField<T> {
    fn from(val: T) -> Self {
        ConfigField {
            _inner: val,
            // name: "Unknown!!".to_string()
        }
    }
}

impl<T> Serialize for ConfigField<T> where T: Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self._inner.serialize(serializer)
    }
}

impl<'a, T> Deserialize<'a> for ConfigField<T> where T:Deserialize<'a> {
    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
        let val = T::deserialize(deserializer)?;
        Ok(
            ConfigField{_inner: val}
        )
    }
}




#[derive(Serialize, Deserialize, Debug)]  // Serializing and deserializing channels will probably have to be reduced to their IDs, not the whole structs
pub struct GuildConfig {
    guild_id: GuildId,
    moderation_channel: ConfigField<Option<ChannelId>>,
    raid_containment_channel: ConfigField<Option<ChannelId>>,
    silence_containment_channel: ConfigField<Option<ChannelId>>,
    log_channel: ConfigField<Option<ChannelId>>,

    member_role: ConfigField<Option<RoleId>>,
    silence_role: ConfigField<Option<RoleId>>,
    new_role: ConfigField<Option<RoleId>>,

    raid_trigger_timespan: ConfigField<u32>,
    // RConfigField<aid> is triggered if n users join within this timespan, in seconds
    raid_trigger_new_user_limit: ConfigField<u32>,
    raid_autoexpiration: ConfigField<u32>,

    // Antispam pressure section
    max_pressure: ConfigField<f64>,
    message_pressure: ConfigField<f64>,
    embed_pressure: ConfigField<f64>,
    character_pressure: ConfigField<f64>,
    newline_pressure: ConfigField<f64>,
    unique_ping_pressure: ConfigField<f64>,
    pressure_decay_per_second: ConfigField<f64>,
    // consider adding custom regex filters for pressure, as well as extra pressure for repeated messages


}

impl GuildConfig {
    fn new(guild_id: GuildId) -> Self {
        GuildConfig {
            guild_id,
            moderation_channel: None.into(),
            raid_containment_channel: None.into(),
            silence_containment_channel: None.into(),
            log_channel: None.into(),
            member_role: None.into(),
            silence_role: None.into(),
            new_role: None.into(),
            raid_trigger_timespan: 90.into(),
            raid_trigger_new_user_limit: 5.into(),
            raid_autoexpiration: 600.into(),
            max_pressure: 60.0.into(),
            message_pressure: 10.0.into(),
            embed_pressure: 8.3.into(),
            character_pressure: 0.00625.into(),
            newline_pressure: 0.714.into(),
            unique_ping_pressure: 2.5.into(),
            pressure_decay_per_second: 8.0.into(),
        }
    }
}


pub(crate) struct GuildShell {
    pub config: GuildConfig,
    current_raid: Option<RaidInfo>,
    last_raid: Option<RaidInfo>,
    active_members: HashMap<UserId, MemberShell>,
    log: HashMap<DateTime<Utc>, String>,
}

impl From<GuildConfig> for GuildShell {
    fn from(config: GuildConfig) -> Self {
        GuildShell {
            config,
            current_raid: None,
            last_raid: None,
            active_members: Default::default(),
            log: Default::default()
        }
    }
}

impl From<GuildId> for GuildShell {
    fn from(guild_id: GuildId) -> Self {
        GuildShell {
            config: GuildConfig::new(guild_id),
            current_raid: None,
            last_raid: None,
            active_members: Default::default(),
            log: Default::default()
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
                log: Default::default()
            }
        )
    }
}


impl GuildShell {
    pub fn get_config(&self) -> &GuildConfig {
        &self.config
    }

    pub fn calculate_message_pressure(&self, msg: &Message) -> f64 {
        let mut pressure: f64 = *self.config.message_pressure;
        pressure += *self.config.embed_pressure * msg.embeds.len() as f64;
        pressure += *self.config.character_pressure * msg.content.len() as f64;
        pressure += *self.config.newline_pressure * msg.content.matches("\n").collect::<String>().len() as f64;
        pressure += *self.config.unique_ping_pressure * msg.mentions.len() as f64;

        pressure
    }

    pub async fn member_joined(&mut self, ctx: &Context, new_member: Member) {
        let new_member_id = new_member.user.id.clone();
        let mut _shell = MemberShell::from(new_member);
        self.active_members.insert(new_member_id, _shell);
        let shell: &mut MemberShell = self.active_members.get_mut(&new_member_id).expect("Unreachable, we just inserted this member");

        if let Some(raid) = &mut self.current_raid {
            raid.raiders.push(new_member_id);
            shell.log("Joined during raid! No automatic role assignment");
        } else {
            if let Some(member_role) = &*self.config.member_role {
                match shell.member.add_role(ctx, member_role).await {
                    Ok(_) => shell.log("Member role added"),
                    Err(e) => shell.log(format!("Adding member role failed! reason: {}", e))
                }
            } else { shell.log("Member role not configured so not assigned.") }
            if let Some(new_role) = &*self.config.new_role {
                match shell.member.add_role(ctx, new_role).await {
                    Ok(_) => shell.log("'New' role added"),
                    Err(e) => shell.log(format!("Adding 'new' role failed! reason: {}", e))
                }
            } else { shell.log("'New'' role not configured so not assigned.") }
        }
    }

    async fn log<S: AsRef<str>>(&mut self, content: S) {
        self.log.insert(Utc::now(), content.as_ref().parse().unwrap());
    }

    async fn ensure_member_shell(&mut self, ctx: &Context, user_id: UserId) -> Result<(), Error> {
        let active_members = &mut self.active_members;

        if active_members.contains_key(&user_id) {
            return Ok(())
        }

        match self.config.guild_id.member(&ctx, user_id).await {
            Ok(m) => {
                let sh = MemberShell::from(m);
                active_members.insert(user_id, sh);
                Ok(())
            },
            Err(e) => {
                println!("Coudlnt get member wtf {}", e);
                return Err(e)
            }
        }
    }

    pub async fn silence_member(&mut self, ctx: &Context, user_id: &UserId) {
        let is_shell = self.ensure_member_shell(&ctx, user_id.clone()).await;

        if is_shell.is_ok() {
            let shell = self.active_members.get_mut(user_id).unwrap();
            if let Some(silence_role) = *self.config.silence_role {
            match shell.member.add_role(&ctx, silence_role).await {
                Ok(_resp) => shell.log("Member silenced successfully"),
                Err(e) => shell.log(format!("Member could not be silenced! {}", e))
            }
            } else {
                shell.log("Silence role is not configured! Could not silence.");
            }
        } else {
            self.log("This member could not be silenced (member shell could not be ensured)").await;
        }

    }

    pub async fn message_created(&mut self, ctx: &Context, message: &Message) {
        if self.ensure_member_shell(&ctx, message.author.id).await.is_ok() {
            let pressure = self.calculate_message_pressure(&message);
            let shell = self.active_members.get_mut(&message.author.id).unwrap();
            let pressure = shell.update_pressure(&self.config.pressure_decay_per_second, &pressure);

            if pressure > *self.config.max_pressure {
                shell.log(format!("Member surpassed the pressure limit of {}", *self.config.max_pressure as i64));
                self.silence_member(&ctx, &message.author.id).await;
            }
        }
    }
}
