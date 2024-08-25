use std::{future::IntoFuture, sync::Arc};

use eyre::Result;
use twilight_model::application::interaction::application_command::CommandOptionValue;
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

    let CommandOptionValue::Boolean(permanence) = interaction.data.options[0].value else {
        let interaction_response =
            create_interaction_response_embed("I couldn't find a value!".to_owned(), true);

        context
            .interaction_client()
            .create_response(interaction.id, &interaction.token, &interaction_response)
            .await?;

        return Ok(());
    };
    let description = if interaction.guild.permanence.read().eq(&permanence) {
        "No change has been applied."
    } else {
        context
            .database
            .update_permanence(interaction.guild.id, permanence)
            .await?;
        context
            .cache
            .update_permanence(interaction.guild.id, permanence);

        if permanence {
            "Empty voice channels (created by me) will not be deleted."
        } else {
            for category_channel_id in interaction.guild.category_channel_ids.read().iter() {
                let Some(category_channel) = context.cache.category_channel(*category_channel_id)
                else {
                    continue;
                };

                for voice_channel_id in category_channel.voice_channel_ids.read().iter() {
                    let Some(voice_channel) = context.cache.voice_channel(*voice_channel_id) else {
                        continue;
                    };

                    if voice_channel.connected_user_ids.read().len().eq(&0) {
                        let command_context = Arc::clone(&context);

                        tokio::spawn(
                            command_context
                                .client
                                .delete_channel(*voice_channel_id)
                                .into_future(),
                        );
                    }
                }
            }

            "Voice channels (created by me) will now deleted when empty."
        }
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
