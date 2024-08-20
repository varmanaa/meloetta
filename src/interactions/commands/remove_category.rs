use std::{collections::HashSet, future::IntoFuture, sync::Arc};

use eyre::Result;
use twilight_model::{
    application::interaction::application_command::CommandOptionValue,
    id::{marker::ChannelMarker, Id},
};
use twilight_util::builder::embed::EmbedBuilder;

use crate::{
    structs::{context::Context, interaction::ApplicationCommandInteraction},
    utilities::interaction::{create_deferred_interaction_response, create_interaction_response_embed},
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
    let description = if interaction
        .guild
        .category_channel_ids
        .read()
        .clone()
        .is_empty()
    {
        "No voice channel categories have been created."
    } else if let Some(category_channel) = context.cache.category_channel(channel_id) {
        let mut channel_ids_to_remove: HashSet<Id<ChannelMarker>> =
            category_channel.voice_channel_ids.read().clone();

        channel_ids_to_remove.insert(category_channel.id);

        if let Some(join_channel_id) = category_channel.join_channel_id.read().clone() {
            channel_ids_to_remove.insert(join_channel_id);
        };

        for channel_id_to_remove in channel_ids_to_remove {
            let command_context = Arc::clone(&context);

            tokio::spawn(
                command_context
                    .client
                    .delete_channel(channel_id_to_remove)
                    .into_future(),
            );
        }

        "I've removed this category!"
    } else {
        "This category is not a voice channel category."
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
