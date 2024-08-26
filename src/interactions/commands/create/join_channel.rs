use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    application::interaction::application_command::CommandOptionValue, channel::ChannelType,
};
use twilight_util::builder::embed::EmbedBuilder;

use crate::{
    structs::{context::Context, interaction::ApplicationCommandInteraction},
    utilities::interaction::{
        create_deferred_interaction_response, create_interaction_response_embed,
    },
};

pub async fn run(context: Arc<Context>, interaction: ApplicationCommandInteraction) -> Result<()> {
    let interaction_response = create_deferred_interaction_response(true);

    context
        .interaction_client()
        .create_response(interaction.id, &interaction.token, &interaction_response)
        .await?;

    let CommandOptionValue::Channel(channel_id) = interaction.data.options[0].value else {
        let interaction_response =
            create_interaction_response_embed("I couldn't find a value!".to_owned(), true);

        context
            .interaction_client()
            .create_response(interaction.id, &interaction.token, &interaction_response)
            .await?;

        return Ok(());
    };
    let description = if let Some(category_channel) = context.cache.category_channel(channel_id) {
        if category_channel.join_channel_id.read().is_some() {
            "This category already has a join channel.".to_owned()
        } else if let Ok(created_join_channel_response) = context
            .client
            .create_guild_channel(interaction.guild.id, "Join to create")
            .kind(ChannelType::GuildVoice)
            .parent_id(category_channel.id)
            .position(0)
            .await
        {
            let created_join_channel = created_join_channel_response.model().await?;
            let created_join_channel_id = created_join_channel.id;

            context
                .database
                .update_join_channel(category_channel.id, Some(created_join_channel_id))
                .await?;
            context
                .cache
                .update_join_channel(category_channel.id, Some(created_join_channel_id));

            format!(
                "<#{}> is now the join voice channel for this category.",
                created_join_channel.id
            )
        } else {
            "I'm unable to create a join voice channel for this category.".to_owned()
        }
    } else {
        "This category is not a voice channel category.".to_owned()
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
