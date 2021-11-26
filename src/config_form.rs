use serenity::model::interactions::Interaction;
use serenity::client::Context;
use crate::guild_shell::{GuildShell, ConfigField};
use serenity::builder::{CreateComponents, CreateSelectMenuOptions};
use serenity::model::id::{ChannelId, RoleId};
use serenity::prelude::SerenityError;
use serde::Serialize;
use rand::prelude::*;


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

    fn set_value(&mut self, new_value: String) -> Result<(), ()>;

    fn add_selection_option(&self, options: &mut CreateSelectMenuOptions) {
        options.create_option(|op| {
            op
                .label(format!("{}", self.get_pretty_name()))
                .description(format!("Set the value for {}", self.get_pretty_name()))
                .value(self.get_selection_key())
        });
    }

    fn setting_updated(&mut self, key: String, value: String) -> Result<bool, ()>{
        if self.get_setting_key() == key {
            self.set_value(value)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn make_config_window(&self, components: &mut CreateComponents) {
        components.create_action_row(|row| {
            row.create_button(|button| {
                button.label(format!("Reset {}", self.get_pretty_name()))
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

    fn set_value(&mut self, new_value: String) -> Result<(), ()> {
        if let Ok(val) = new_value.parse() {
            self._inner = val;
            Ok(())
        } else {
            Err(())
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

    fn set_value(&mut self, new_value: String) -> Result<(), ()> {
        todo!()
    }
}

impl Configurable for ConfigField<Option<RoleId>> {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_pretty_name(&self) -> &String {
        &self.name
    }

    fn set_value(&mut self, new_value: String) -> Result<(), ()> {
        todo!()
    }
}

impl Configurable for ConfigField<u32> {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_pretty_name(&self) -> &String {
        &self.name
    }

    fn set_value(&mut self, new_value: String) -> Result<(), ()> {
        todo!()
    }
}


impl GuildShell {
    pub async fn handle_interaction(&mut self, ctx: &Context, interaction: &Interaction) -> Result<(), SerenityError> {
        println!("Interaction handling: {}", serde_yaml::to_string(&interaction).unwrap());

        let id = self.config_component_id.unwrap_or(rand::thread_rng().gen());
        self.config_component_id = Some(id);

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
                                    c.create_action_row(|row| {
                                        row.create_select_menu(|menu| {
                                            menu.options(|opts| {
                                                for field in self.config.get_configurable_fields() {
                                                    field.add_selection_option(opts);
                                                }
                                                opts
                                            })
                                        }
                                            .custom_id(id))
                                    })
                                })
                        })
                    }).await.unwrap();
                }
            }
            Interaction::MessageComponent(component) => {
                if component.message.clone().regular().unwrap().guild_id.unwrap() == self.config.guild_id {
                    let values = &component.data.values;
                    if values[0].starts_with("select_") {
                        for f in self.config.get_configurable_fields() {
                            if *f.get_selection_key() == values[0] {
                                component.create_interaction_response(&ctx, |resp| {
                                    resp.interaction_response_data(|d| {
                                        d.create_embed(|e| e
                                            .title(format!("Set {}", f.get_pretty_name()))
                                            .description(format!("Set value for {} or reset it", f.get_pretty_name())))
                                            .components(|comp| {f.make_config_window(comp); comp})
                                    })
                                }).await?;
                            }
                        }
                    }
                }
            }
            _ => ()
        }
        Ok(())
    }
}

