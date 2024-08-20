use std::sync::Arc;

use eyre::Result;
use twilight_model::id::{marker::UserMarker, Id};
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

    let user_id: Id<UserMarker> = interaction.data.values.iter().nth(0).unwrap().parse()?;
    let description = if interaction
        .voice_channel
        .connected_user_ids
        .read()
        .contains(&user_id)
    {
        context
            .client
            .update_guild_member(interaction.voice_channel.guild_id, user_id)
            .channel_id(None)
            .await?;

        format!("I've removed <@{user_id}> from this voice channel!")
    } else {
        format!("<@{user_id}> isn't in this voice channel!")
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
