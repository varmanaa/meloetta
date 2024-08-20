use std::sync::Arc;

use eyre::{eyre, Result};
use twilight_model::{
    application::interaction::{Interaction, InteractionType},
    channel::message::{Component, MessageFlags},
    guild::Permissions,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::structs::{
    cache::{CachedGuild, CachedVoiceChannel},
    context::Context,
};

pub fn create_deferred_interaction_response(ephemeral: bool) -> InteractionResponse {
    let mut data_builder = InteractionResponseDataBuilder::new();

    if ephemeral {
        data_builder = data_builder.flags(MessageFlags::EPHEMERAL);
    }

    let data = data_builder.build();
    let interaction_response = InteractionResponse {
        data: Some(data),
        kind: InteractionResponseType::DeferredChannelMessageWithSource,
    };

    interaction_response
}

pub fn create_interaction_response_embed(
    description: String,
    ephemeral: bool,
) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(description)
        .build();
    let mut data_builder = InteractionResponseDataBuilder::new().embeds(vec![embed]);

    if ephemeral {
        data_builder = data_builder.flags(MessageFlags::EPHEMERAL);
    }

    let data = data_builder.build();
    let interaction_response = InteractionResponse {
        data: Some(data),
        kind: InteractionResponseType::ChannelMessageWithSource,
    };

    interaction_response
}

pub fn create_interaction_response_modal(
    custom_id: String,
    components: Vec<Component>,
    title: String,
) -> InteractionResponse {
    let data = InteractionResponseDataBuilder::new()
        .custom_id(custom_id)
        .components(components)
        .title(title)
        .build();
    let interaction_response = InteractionResponse {
        data: Some(data),
        kind: InteractionResponseType::Modal,
    };

    interaction_response
}

pub fn create_interaction_response_select(
    components: Vec<Component>,
    ephemeral: bool,
) -> InteractionResponse {
    let mut data_builder = InteractionResponseDataBuilder::new().components(components);

    if ephemeral {
        data_builder = data_builder.flags(MessageFlags::EPHEMERAL);
    }

    let data = data_builder.build();
    let interaction_response = InteractionResponse {
        data: Some(data),
        kind: InteractionResponseType::ChannelMessageWithSource,
    };

    interaction_response
}

pub fn check_interaction(
    context: &Arc<Context>,
    interaction: &Interaction,
) -> Result<(Option<Arc<CachedGuild>>, Option<Arc<CachedVoiceChannel>>)> {
    let Some(guild_id) = interaction.guild_id else {
        return Err(eyre!("I may only be used in servers!"));
    };

    match interaction.kind {
        InteractionType::ApplicationCommand => {
            let Some(guild) = context.cache.guild(guild_id) else {
                return Err(eyre!("Please kick and re-invite me!"));
            };

            if interaction.member.as_ref().is_some_and(|member| {
                member.permissions.is_some_and(|permissions| {
                    !permissions.contains(Permissions::ADMINISTRATOR | Permissions::MANAGE_GUILD)
                })
            }) {
                return Err(eyre!("You need either the **Administrator** or **Manage Server** permissions to use this command!"));
            }

            Ok((Some(guild), None))
        }
        InteractionType::MessageComponent | InteractionType::ModalSubmit => {
            if interaction.app_permissions.is_some_and(|permissions| {
                !permissions.contains(
                    Permissions::CONNECT
                        | Permissions::EMBED_LINKS
                        | Permissions::MANAGE_CHANNELS
                        | Permissions::MANAGE_ROLES
                        | Permissions::MOVE_MEMBERS
                        | Permissions::SEND_MESSAGES
                        | Permissions::VIEW_CHANNEL,
                )
            }) {
                return Err(eyre!("I don't have the right permissions in this channel!"));
            }

            let Some(channel) = interaction.channel.as_ref() else {
                return Err(eyre!("I don't recognize this channel!"));
            };
            let Some(voice_channel) = context.cache.voice_channel(channel.id) else {
                return Err(eyre!("I don't recognize this voice channel!"));
            };

            Ok((None, Some(voice_channel)))
        }
        _ => Err(eyre!("I don't recognize this interaction type!")),
    }
}
