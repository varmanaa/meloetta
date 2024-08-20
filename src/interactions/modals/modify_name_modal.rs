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

    let name = interaction.data.components[0].components[0]
        .value
        .to_owned()
        .unwrap();
    let description = context
        .client
        .update_channel(interaction.voice_channel.id)
        .name(&name)
        .await
        .map_or(
            "I'm unable to rename the voice channel right now, try again in ten minutes",
            |_| "I've changed the name!",
        )
        .to_owned();
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
