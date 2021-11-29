use serenity::builder::{CreateApplicationCommand, CreateComponents, CreateSelectMenuOptions};
use serenity::client::Context;
use serenity::model::channel::{ChannelType, GuildChannel};
use serenity::model::guild::Role;
use serenity::model::id::{ChannelId, RoleId};
use serenity::model::interactions::Interaction;
use serenity::model::interactions::message_component::ButtonStyle;
use serenity::model::prelude::application_command::ApplicationCommandOptionType;
use serenity::prelude::SerenityError;

use crate::error_handling::BetterHandle;
use crate::guild_shell::{ConfigField, GuildShell};

pub trait Configurable {
    fn get_name(&self) -> &String;
    fn get_pretty_name(&self) -> &String;
    fn get_selection_key(&self) -> String {
        format!("select_{}", self.get_name())
    }
    fn get_setting_key(&self) -> String {
        format!("set_{}", self.get_name())
    }
    fn get_reset_key(&self) -> String {
        format!("set_{}", self.get_name())
    }

    fn set_value(&mut self, new_value: String) -> Result<(), String>;

    fn add_selection_option(&self, options: &mut CreateSelectMenuOptions) {
        options.create_option(|op| {
            op
                .label(format!("{}", self.get_pretty_name()))
                .description(format!("Set the value for {}", self.get_pretty_name()))
                .value(self.get_selection_key())
        });
    }

    fn setting_updated(&mut self, key: String, value: String) -> Result<bool, String> {
        if self.get_setting_key() == key {
            self.set_value(value)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn make_config_window(&self, components: &mut CreateComponents, _roles: Vec<&Role>, _channels: Vec<&GuildChannel>) {
        components.create_action_row(|row| {
            row.create_button(|button| {
                button
                    .label(format!("Reset {}", self.get_pretty_name()))
                    .style(ButtonStyle::Secondary)
                    .custom_id(self.get_name())
            })
        });
    }

    fn get_slash_command_type(&self) -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::String
    }

    fn add_slash_command_subcommand(&self, cmd: &mut CreateApplicationCommand) {
        cmd
            .create_option(|opt| {
                opt.name(self.get_name())
                    .description(format!("Set the value for {}", self.get_pretty_name()))
                    .kind(ApplicationCommandOptionType::SubCommand)
                    .create_sub_option(|sub| {
                        sub.name("value").description("Value to set (leave empty to reset to default)")
                            .kind(self.get_slash_command_type())
                    })
            });

        /*.name(self.get_name())
        .description(format!("Set the value for {}", self.get_pretty_name()))
        .kind(ApplicationCommandOptionType::SubCommand);
        .create_sub_option(|o| {
            o.name(format!("{}", self.get_name())).description("The value to set").kind(ApplicationCommandOptionType::String).required(true)
        })*/
    }
}


impl Configurable for ConfigField<f64> {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_pretty_name(&self) -> &String {
        &self.name
    }

    fn set_value(&mut self, new_value: String) -> Result<(), String> {
        match new_value.parse() {
            Ok(val) => {
                self._inner = val;
                Ok(())
            }
            Err(e) => Err(e.to_string())
        }
    }

    fn get_slash_command_type(&self) -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::Number
    }
}

impl Configurable for ConfigField<Option<ChannelId>> {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_pretty_name(&self) -> &String {
        &self.name
    }

    fn get_slash_command_type(&self) -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::Channel
    }

    fn set_value(&mut self, new_value: String) -> Result<(), String> {
        if new_value == "" {
            self._inner = None;
            return Ok(());
        }

        match new_value.parse::<u64>() {
            Ok(to_num) => {
                let channel_id = ChannelId::from(to_num);
                self._inner = Some(channel_id);
                return Ok(());
            }
            Err(e) => Err(format!("{} is not a valid id: {}", new_value, e))
        }
    }


    fn make_config_window(&self, components: &mut CreateComponents, _roles: Vec<&Role>, channels: Vec<&GuildChannel>) {
        /*components.create_action_row(|row| {
            row.create_button(|button| {
                button
                    .label(format!("Reset {}", self.get_pretty_name()))
                    .style(ButtonStyle::Secondary)
                    .custom_id(self.get_name())
            })
        });*/

        components.create_action_row(|row| {
            row.create_select_menu(|menu| {
                menu.custom_id(self.get_setting_key())
                    .options(|op| {
                        for c in channels {
                            op.create_option(|o| {
                                o.label(&c.name).description(format!("Pick {}", &c.name)).value(c.id)
                            });
                        }
                        op
                    })
            })
        });
    }
}

impl Configurable for ConfigField<Option<RoleId>> {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_pretty_name(&self) -> &String {
        &self.name
    }

    fn get_slash_command_type(&self) -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::Role
    }

    fn set_value(&mut self, new_value: String) -> Result<(), String> {
        if new_value == "" {
            self._inner = None;
            return Ok(());
        }

        if let Ok(to_num) = new_value.parse::<u64>() {
            let role_id = RoleId::from(to_num);
            self._inner = Some(role_id);
            return Ok(());

        }
        return Err("Not a valid id".into());
    }

    fn make_config_window(&self, components: &mut CreateComponents, roles: Vec<&Role>, _channels: Vec<&GuildChannel>) {
        components.create_action_row(|row| {
            row.create_select_menu(|menu| {
                menu.custom_id(self.get_setting_key())
                    .options(|op| {
                        for r in roles {
                            op.create_option(|o| {
                                o.label(&r.name).description(format!("Choose {}", &r.name)).value(r.id)
                            });
                        }
                        op
                    })
            })
        });
    }
}

impl Configurable for ConfigField<u32> {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_pretty_name(&self) -> &String {
        &self.name
    }

    fn set_value(&mut self, _new_value: String) -> Result<(), String> {
        todo!()
    }

    fn get_slash_command_type(&self) -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::Integer
    }
}


impl GuildShell {
    fn add_selection_components(&mut self, components: &mut CreateComponents) {
        components.create_action_row(|row| {
            row.create_select_menu(|menu| {
                menu.options(|opts| {
                    for field in self.config.get_configurable_fields() {
                        field.add_selection_option(opts);
                    }
                    opts
                })
            }
                .custom_id("config_selection"))
        });
    }

    pub async fn dump_logs(&mut self, ctx: &Context) -> Result<(), SerenityError> {
        if let Some(ch) = *self.config.log_channel {
            if let Some(serverlog) = self._log.dump() {
                ch.send_message(&ctx, |msg| {
                    msg.add_embed(|e| {
                        e.title("Server info").description(format!("```{}```", serverlog))
                    })
                }).await?;   //.dexpect("Failed to send server log  to the log channel", &mut self._log);
                self._log.clear();
            }

            for m in self.active_members.values_mut() {
                if let Some(memberlogs) = m._log.dump() {
                    ch.send_message(&ctx, |msg| {
                        msg.add_embed(|e| {
                            e.title(format!("Member {}", m.member.nick.as_ref().unwrap_or(&m.member.user.name))).description(format!("<@{}>\n```{}```", m.member.user.id, memberlogs))
                        })
                    }).await?;   // .dexpect("Failed to send message to the log channel", &mut self._log);
                    m._log.clear();
                }
            }
            Ok(())
        } else {
            Err(SerenityError::Other("No log channel configured"))
        }
    }

    pub async fn handle_interaction(&mut self, ctx: &Context, interaction: &Interaction) -> Result<(), SerenityError> {
        println!("Interaction handling: {}", serde_yaml::to_string(&interaction).unwrap());

        match interaction {
            Interaction::ApplicationCommand(c) => {
                if c.guild_id != Some(self.config.guild_id) {
                    // println!("Interaction from guild {}, this is shell for {}", c.guild_id.unwrap(), self.config.guild_id);
                    return Ok(());
                }
            }
            Interaction::MessageComponent(c) => {
                if c.guild_id != Some(self.config.guild_id) {
                    return Ok(());
                }
            }
            _ => println!("Unknown interaction type?? {}", serde_yaml::to_string(&interaction.kind()).unwrap())
        }

        match interaction {
            Interaction::ApplicationCommand(command) => {
                match command.data.name.as_ref() {
                    "config" => {
                        command.create_interaction_response(&ctx, |resp| {
                            resp.interaction_response_data(|d| {
                                d.create_embed(|e|
                                    e
                                        .title("Bussy configuration")
                                        .description("Pick which setting do you want to configure. That will bring you onto the next screen."))
                                    .components(|c| {
                                        self.add_selection_components(c);
                                        c
                                    })
                            })
                        }).await.dexpect("Failed to send interaction response", &mut self._log);
                    }
                    "change" => {
                        let fields = self.config.get_configurable_fields();
                        let name = &command.data.options[0].name;
                        let value = &command.data.options[0].options[0].value;


                        if let Some(field) = fields.into_iter().find(|f| f.get_name() == name) {
                            let res = match value {
                                Some(s) => field.set_value(s.as_str().unwrap_or("").to_string()),
                                None => field.set_value("".to_string())
                            };

                            let title = match res {
                                Ok(()) => format!("Change successful"),
                                Err(_) => format!("No.")
                            };
                            let body = match res {
                                Ok(()) => format!("{} changed successfully", name),
                                Err(e) => format!("{}", e)
                            };

                            command.create_interaction_response(&ctx, |resp| {
                                resp.interaction_response_data(|data| {
                                    data.create_embed(|e| {
                                        e.title(title)
                                            .description(body)
                                    })
                                })
                            }).await?;
                        }
                    }
                    _ => println!("Unknown application command {}!", command.data.name)
                }
            }
            Interaction::MessageComponent(component) => {
                //if component.message.clone().regular().unwrap() == self.config.guild_id {
                let guild = ctx.http.get_guild(self.config.guild_id.into()).await.dexpect("Couldn't fetch guild", &mut self._log);

                let roles: Vec<&Role> = guild.roles.values().collect();
                let channels = guild.channels(&ctx).await.dexpect("Couldn't retrieve guild channels", &mut self._log);
                let channels = channels.values().filter(|c| c.kind == ChannelType::Text).collect();

                let custom_id = &component.data.custom_id;
                let values = &component.data.values;
                if custom_id == "config_selection" {
                    let action = &values[0];
                    for f in self.config.get_configurable_fields() {
                        if &f.get_selection_key() == action {
                            // println!("Found selection!!");
                            component.create_interaction_response(&ctx, |resp| {
                                resp.interaction_response_data(|d| {
                                    d
                                        .create_embed(|e| e
                                            .title(format!("Set {}", f.get_pretty_name()))
                                            .description(format!("Set value for {} or reset it", f.get_pretty_name()))
                                        )
                                        .components(|comp| {
                                            f.make_config_window(comp, roles, channels);
                                            comp
                                        })
                                })
                            }).await.dexpect("Failed to respond to interaction", &mut self._log);
                            break;
                        }
                    }
                } else if custom_id.starts_with("set_") {
                    let fl = self.config.get_configurable_fields().into_iter().find(|field| &field.get_setting_key() == custom_id);
                    if let Some(f) = fl {
                        let res = f.set_value(component.data.values[0].clone());
                        let to_say = match res {
                            Ok(()) => "Value changed successfully!".to_string(),
                            Err(e) => format!("Couldn't set value: {}", e)
                        };
                        component.create_interaction_response(&ctx, |resp| {
                            resp.interaction_response_data(|data| {
                                data.create_embed(|e| {
                                    e.title("Value change").description(to_say)
                                })
                                    .components(|comp| {
                                        self.add_selection_components(comp);
                                        comp
                                    })
                            })
                        }).await?;
                    } else {
                        println!("Nonexisting field");
                    }
                }
            }
            _ => ()
        }
        Ok(())
    }
}

