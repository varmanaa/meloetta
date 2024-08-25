pub mod permanence;
pub mod privacy;
pub mod show;

use std::{mem::replace, sync::Arc};

use eyre::Result;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue,
};

use crate::{
    structs::{context::Context, interaction::ApplicationCommandInteraction},
    utilities::interaction::create_interaction_response_embed,
};

pub async fn run(
    context: Arc<Context>,
    mut interaction: ApplicationCommandInteraction,
) -> Result<()> {
    let command_options = interaction.data.options.clone();
    let CommandDataOption { name, value } = command_options.iter().nth(0).unwrap();
    let CommandOptionValue::SubCommand(options) = value.clone() else {
        let interaction_response =
            create_interaction_response_embed("I couldn't find a value!".to_owned(), true);

        context
            .interaction_client()
            .create_response(interaction.id, &interaction.token, &interaction_response)
            .await?;

        return Ok(());
    };
    let _ = replace(&mut interaction.data.options, options);

    match name.as_str() {
        "permanence" => permanence::run(context, interaction).await?,
        "privacy" => privacy::run(context, interaction).await?,
        "show" => show::run(context, interaction).await?,
        _ => {
            let interaction_response = create_interaction_response_embed(
                format!("I don't have a subcommand with the name \"{name}\"!"),
                true,
            );

            context
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &interaction_response)
                .await?;
        }
    }

    Ok(())
}
