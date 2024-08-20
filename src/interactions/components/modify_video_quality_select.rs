use std::sync::Arc;

use eyre::Result;
use twilight_model::channel::VideoQualityMode;
use twilight_util::builder::embed::EmbedBuilder;

use crate::{
    structs::{context::Context, interaction::MessageComponentInteraction},
    utilities::interaction::create_deferred_interaction_response,
};

pub async fn run(context: Arc<Context>, interaction: MessageComponentInteraction) -> Result<()> {
    let interaction_response = create_deferred_interaction_response(true);

    context
        .interaction_client()
        .create_response(interaction.id, &interaction.token, &interaction_response)
        .await?;

    let video_quality_option = interaction.data.values.clone().into_iter().nth(0).unwrap();
    let (mode, text) = match video_quality_option.as_str() {
        "2" => (VideoQualityMode::Full, "720p"),
        _ => (VideoQualityMode::Auto, "Auto"),
    };

    context
        .client
        .update_channel(interaction.voice_channel.id)
        .video_quality_mode(mode)
        .await?;

    let description =
        format!("I've modified the video quality to **{text}** in this voice channel!");
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
