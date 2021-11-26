use serenity::model::interactions::Interaction;
use serenity::client::Context;
use crate::guild_shell::{GuildShell, ConfigField};
use serenity::builder::{CreateComponents, CreateSelectMenuOptions};
use serenity::model::id::{ChannelId, RoleId};
use serenity::prelude::SerenityError;
use serenity::model::interactions::message_component::ButtonStyle;
use serenity::model::channel::{ChannelType, GuildChannel};
use serenity::model::guild::{Role};


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

    fn setting_updated(&mut self, key: String, value: String) -> Result<bool, String>{
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

}

impl Configurable for ConfigField<Option<ChannelId>> {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_pretty_name(&self) -> &String {
        &self.name
    }

    fn set_value(&mut self, new_value: String) -> Result<(), String> {
        if new_value == "" {
            self._inner = None;
            return Ok(())
        }

        if let Ok(to_num) = new_value.parse::<u64>() {
            let channel_id = ChannelId::from(to_num);
                self._inner = Some(channel_id);
                return Ok(())
        }
        return Err("Not a valid id".into())
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

    fn set_value(&mut self, new_value: String) -> Result<(), String> {
        if new_value == "" {
            self._inner = None;
            return Ok(())
        }

        if let Ok(to_num) = new_value.parse::<u64>() {
            let role_id = RoleId::from(to_num);
                self._inner = Some(role_id);
                return Ok(())

        }
        return Err("Not a valid id".into())
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

    pub async fn handle_interaction(&mut self, ctx: &Context, interaction: &Interaction) -> Result<(), SerenityError> {
        println!("Interaction handling: {}", serde_yaml::to_string(&interaction).unwrap());

        match interaction {
             Interaction::ApplicationCommand(c) => {
                 if c.guild_id != Some(self.config.guild_id) {
                     println!("Interaction from guild {}, this is shell for {}", c.guild_id.unwrap(), self.config.guild_id);
                     return Ok(())
                 }
             }
            Interaction::MessageComponent(c) => {
                if c.guild_id != Some(self.config.guild_id) {
                    return Ok(())
                }
            }
            _ => println!("Unknown interaction type?? {}", serde_yaml::to_string( &interaction.kind()).unwrap())
        }

        match interaction {
            Interaction::ApplicationCommand(command) => {
                if command.data.name == "config" {
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
                    }).await.unwrap();
                }
            }
            Interaction::MessageComponent(component) => {
                //if component.message.clone().regular().unwrap() == self.config.guild_id {
                let guild = ctx.http.get_guild(self.config.guild_id.into()).await.expect("Guild must be retrievable");
                // let guild = ctx.cache.guild(self.config.guild_id).await.expect("Guild must be retrievable");

                let roles: Vec<&Role> = guild.roles.values().collect();
                let channels= guild.channels(&ctx).await.expect("Guild channels must be retrievable");
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
                            }).await.expect("Responding to interaction should work");
                            break;
                        }
                    }
                }
                else if custom_id.starts_with("set_") {
                    for f in self.config.get_configurable_fields() {
                        if &f.get_setting_key() == custom_id {
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
                            break
                        }
                    }
                }
            }
            _ => ()
        }
        Ok(())
    }
}

