use std::sync::Arc;

use eyre::Result;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_util::builder::embed::EmbedBuilder;

use crate::{
    structs::{context::Context, interaction::ApplicationCommandInteraction},
    utilities::{
        constants::{PANEL_MESSAGE_COMPONENTS, PANEL_MESSAGE_EMBED},
        interaction::{create_deferred_interaction_response, create_interaction_response_embed},
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
    let description = if let Some(voice_channel) = context.cache.voice_channel(channel_id) {
        let panel_message_id = voice_channel.panel_message_id.read().clone();
        let is_panel_message_valid = if let Some(panel_message_id) = panel_message_id {
            if let Ok(response) = context
                .client
                .message(voice_channel.id, panel_message_id)
                .await
            {
                response.model().await.is_ok()
            } else {
                false
            }
        } else {
            false
        };

        if is_panel_message_valid {
            format!(
                "One already exists at https://discord.com/channels/{}/{}/{}!",
                interaction.guild.id,
                voice_channel.id,
                panel_message_id.unwrap()
            )
        } else {
            let new_panel_message_id = Some(
                context
                    .client
                    .create_message(voice_channel.id)
                    .components(&*PANEL_MESSAGE_COMPONENTS)
                    .embeds(&[PANEL_MESSAGE_EMBED.clone()])
                    .await?
                    .model()
                    .await?
                    .id,
            );

            context
                .database
                .update_panel_message(voice_channel.id, new_panel_message_id)
                .await?;
            context
                .cache
                .update_panel_message(voice_channel.id, new_panel_message_id);

            format!("I've made a new panel message!")
        }
    } else {
        "I didn't create this voice channel.".to_owned()
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
