use std::sync::Arc;

use eyre::Result;
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

    let description = if context
        .cache
        .voice_channel_owner(interaction.voice_channel.guild_id, interaction.user_id)
        .is_some()
    {
        "You already own a voice channel!"
    } else if interaction.voice_channel.owner_id.read().is_some() {
        "You may only claim voice channels without owners."
    } else {
        context
            .database
            .update_voice_channel_owner(interaction.voice_channel.id, Some(interaction.user_id))
            .await?;
        context
            .cache
            .update_voice_channel_owner(interaction.voice_channel.id, Some(interaction.user_id));

        "You now own this voice channel!"
    };
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(description.to_owned())
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
