use std::sync::Arc;

use eyre::Result;
use twilight_util::builder::embed::EmbedBuilder;

use crate::{
    structs::{context::Context, interaction::ModalSubmitInteraction},
    utilities::interaction::create_deferred_interaction_response,
};

pub async fn run(context: Arc<Context>, interaction: ModalSubmitInteraction) -> Result<()> {
    let interaction_response = create_deferred_interaction_response(true);

    context
        .interaction_client()
        .create_response(interaction.id, &interaction.token, &interaction_response)
        .await?;

    let bitrate_value = interaction.data.components[0].components[0]
        .value
        .to_owned()
        .unwrap();
    let description = if let Ok(bitrate) = bitrate_value.parse::<u32>() {
        if (8..=96).contains(&bitrate) {
            context
                .client
                .update_channel(interaction.voice_channel.id)
                .bitrate(u32::from(bitrate * 1000))
                .await?;

            format!("I've changed the bitrate to {bitrate}kbps!")
        } else {
            "The bitrate must be between 8 and 96, inclusive.".to_owned()
        }
    } else {
        format!("{bitrate_value} is not a valid integer!")
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
