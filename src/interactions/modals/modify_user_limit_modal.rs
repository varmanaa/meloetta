use std::sync::Arc;

use eyre::Result;
use twilight_util::builder::embed::EmbedBuilder;

use crate::{structs::{context::Context, interaction::ModalSubmitInteraction}, utilities::interaction::create_deferred_interaction_response};

pub async fn run(context: Arc<Context>, interaction: ModalSubmitInteraction) -> Result<()> {
    let interaction_response = create_deferred_interaction_response(true);

    context
        .interaction_client()
        .create_response(interaction.id, &interaction.token, &interaction_response)
        .await?;

    let user_limit_value = interaction.data.components[0].components[0]
        .value
        .to_owned()
        .unwrap();
    let description = if let Ok(user_limit) = user_limit_value.parse::<u16>() {
        context
            .client
            .update_channel(interaction.voice_channel.id)
            .user_limit(user_limit)
            .await
            .map_or(
                "The user limit must be between 0 and 99, inclusive.".to_owned(),
                |_| format!("I've changed the user limit to {user_limit}!"),
            )
    } else {
        format!("{user_limit_value} is not a valid integer!")
    };
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(description)
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
