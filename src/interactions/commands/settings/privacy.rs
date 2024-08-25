use std::{future::IntoFuture, sync::Arc};

use eyre::Result;
use twilight_model::{
    application::interaction::application_command::CommandOptionValue,
    channel::permission_overwrite::PermissionOverwriteType as ChannelPermissionOverwriteType,
    guild::Permissions,
    http::permission_overwrite::{
        PermissionOverwrite as HttpPermissionOverwrite,
        PermissionOverwriteType as HttpPermissionOverwriteType,
    },
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

    let CommandOptionValue::String(privacy_option) = interaction.data.options[0].clone().value
    else {
        let interaction_response =
            create_interaction_response_embed("I couldn't find a value!".to_owned(), true);

        context
            .interaction_client()
            .create_response(interaction.id, &interaction.token, &interaction_response)
            .await?;

        return Ok(());
    };
    let description = if interaction.guild.privacy.read().clone().eq(&privacy_option) {
        "No change has been applied."
    } else {
        context
            .database
            .update_privacy(interaction.guild.id, privacy_option.clone())
            .await?;
        context
            .cache
            .update_privacy(interaction.guild.id, privacy_option.clone());

        for category_channel_id in interaction.guild.category_channel_ids.read().iter() {
            let Some(category_channel) = context.cache.category_channel(*category_channel_id)
            else {
                continue;
            };
            let (mut everyone_allow, mut everyone_deny) = category_channel
                .permission_overwrites
                .read()
                .iter()
                .find(|permission_overwrite| {
                    permission_overwrite.id.eq(&interaction.guild.id.cast())
                        && permission_overwrite
                            .kind
                            .eq(&ChannelPermissionOverwriteType::Role)
                })
                .map_or(
                    (Permissions::empty(), Permissions::empty()),
                    |permission_overwrite| (permission_overwrite.allow, permission_overwrite.deny),
                );

            everyone_allow.remove(Permissions::CONNECT | Permissions::VIEW_CHANNEL);
            everyone_deny.remove(Permissions::CONNECT | Permissions::VIEW_CHANNEL);

            if privacy_option.eq("invisible") {
                everyone_deny.insert(Permissions::VIEW_CHANNEL);
            } else if privacy_option.eq("locked") {
                everyone_deny.insert(Permissions::CONNECT);
            }

            let command_context = Arc::clone(&context);

            tokio::spawn(
                command_context
                    .client
                    .update_channel_permission(
                        category_channel.id,
                        &HttpPermissionOverwrite {
                            allow: Some(everyone_allow),
                            deny: Some(everyone_deny),
                            id: interaction.guild.id.cast(),
                            kind: HttpPermissionOverwriteType::Role,
                        },
                    )
                    .into_future(),
            );

            if let Some(join_channel_id) = category_channel.join_channel_id.read().clone() {
                tokio::spawn(
                    command_context
                        .client
                        .update_channel_permission(
                            join_channel_id,
                            &HttpPermissionOverwrite {
                                allow: Some(everyone_allow),
                                deny: Some(everyone_deny),
                                id: interaction.guild.id.cast(),
                                kind: HttpPermissionOverwriteType::Role,
                            },
                        )
                        .into_future(),
                );
            };
        }

        if privacy_option.eq("invisible") {
            "New voice channels, by default, will be invisible."
        } else if privacy_option.eq("locked") {
            "New voice channels, by default, will be locked and visible."
        } else {
            "New voice channels, by default, will be unlocked and visible."
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
