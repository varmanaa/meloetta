use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    application::interaction::application_command::CommandOptionValue,
    channel::{
        permission_overwrite::{
            PermissionOverwrite as ChannelPermissionOverwrite,
            PermissionOverwriteType as ChannelPermissionOverwriteType,
        },
        ChannelType,
    },
    guild::Permissions,
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

    let CommandOptionValue::String(name) = interaction.data.options[0].value.clone() else {
        let interaction_response =
            create_interaction_response_embed("I couldn't find a value!".to_owned(), true);

        context
            .interaction_client()
            .create_response(interaction.id, &interaction.token, &interaction_response)
            .await?;

        return Ok(());
    };
    let everyone_deny = if interaction.guild.privacy.read().eq("invisible") {
        Permissions::VIEW_CHANNEL
    } else {
        Permissions::empty()
    };
    let description = if interaction.guild.category_channel_ids.read().len() > 3 {
        "I'm only allowing a maximum of three voice channel categories in this server!".to_owned()
    } else if let Ok(created_category_channel_response) = context
        .client
        .create_guild_channel(interaction.guild.id, &name)
        .kind(ChannelType::GuildCategory)
        .permission_overwrites(&[
            ChannelPermissionOverwrite {
                allow: Permissions::empty(),
                deny: everyone_deny,
                id: interaction.guild.id.cast(),
                kind: ChannelPermissionOverwriteType::Role,
            },
            ChannelPermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL,
                deny: Permissions::empty(),
                id: interaction.guild.bot_role_id.cast(),
                kind: ChannelPermissionOverwriteType::Role,
            },
        ])
        .await
    {
        let created_category_channel = created_category_channel_response.model().await?;
        let created_category_channel_id = created_category_channel.id;
        let created_join_channel_id = if let Ok(res) = context
            .client
            .create_guild_channel(interaction.guild.id, "Join to create")
            .kind(ChannelType::GuildVoice)
            .parent_id(created_category_channel_id)
            .permission_overwrites(&[ChannelPermissionOverwrite {
                allow: Permissions::CONNECT | Permissions::VIEW_CHANNEL,
                deny: Permissions::empty(),
                id: interaction.guild.bot_role_id.cast(),
                kind: ChannelPermissionOverwriteType::Role,
            }])
            .position(0)
            .await
        {
            Some(res.model().await?.id)
        } else {
            None
        };

        context
            .database
            .insert_category_channel(
                created_category_channel_id,
                interaction.guild.id,
                created_join_channel_id,
            )
            .await?;
        context.cache.insert_category_channel(
            interaction.guild.id,
            created_category_channel_id,
            created_join_channel_id,
            created_category_channel
                .permission_overwrites
                .unwrap_or_default(),
            Vec::new(),
        );

        if created_join_channel_id.is_some() {
            format!("I've created <#{created_category_channel_id}>!")
        } else {
            "I'm unable to create a join voice channel for this category. Delete a channel and run the ```/create join-channel``` command.".to_owned()
        }
    } else {
        "I'm unable to create a voice channel category.".to_owned()
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
