use serenity::model::interactions::Interaction;
use serenity::client::Context;



pub async fn handle_interaction(_ctx: Context, _interaction: Interaction) {
/*
    let pick_setting_action_row = |mut comp: CreateComponents| {
        comp.create_action_row(|row| {
            row.create_select_menu(|menu| {
                menu.options(|options| {
                    options.create_option(|op| {
                        op.label("Max pressure")
                            .description("Maximum pressure one user can exert before being silenced")
                            .value("max_pressure")
                    })
                })
            })
        })
    };

    if let Interaction::ApplicationCommand(command) = interaction {
        if command.data.name == "config" {
            command.create_interaction_response(&ctx, |resp| {
                resp.interaction_response_data(|d| {
                    d.create_embed(|e|
                        e
                            .title("Bussy configuration")
                            .description("Pick which setting do you want to configure. That will bring you onto the next screen."))
                        .components()
                })
            })
        }
    }
    */

}


