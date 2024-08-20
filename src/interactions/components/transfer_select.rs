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

    let user = interaction
        .data
        .resolved
        .unwrap()
        .users
        .values()
        .cloned()
        .nth(0)
        .unwrap();
    let user_id = user.id;
    let description = if user.bot {
        format!("You can't transfer this voice channel to this user!")
    } else if context
        .cache
        .voice_channel_owner(interaction.voice_channel.guild_id, user_id)
        .is_some()
    {
        format!("<@{user_id}> already owns a voice channel!")
    } else {
        context
            .database
            .update_voice_channel_owner(interaction.voice_channel.id, Some(user_id))
            .await?;
        context
            .cache
            .update_voice_channel_owner(interaction.voice_channel.id, Some(user_id));

        format!("<@{user_id}> now owns this voice channel!")
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
