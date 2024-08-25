use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    channel::permission_overwrite::{
        PermissionOverwrite as ChannelPermissionOverwrite,
        PermissionOverwriteType as ChannelPermissionOverwriteType,
    },
    guild::Permissions,
    id::{marker::GenericMarker, Id},
};
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

    let privacy_option = interaction.data.values.clone().into_iter().nth(0).unwrap();
    let mut voice_channel_permission_overwrites = interaction
        .voice_channel
        .permission_overwrites
        .read()
        .clone();
    let current_privacy_option = voice_channel_permission_overwrites
        .iter()
        .find(|permission_overwrite| {
            permission_overwrite
                .id
                .eq(&interaction.voice_channel.guild_id.cast())
                && permission_overwrite
                    .kind
                    .eq(&ChannelPermissionOverwriteType::Role)
        })
        .map_or("unlocked", |everyone_permission_overwrite| {
            if everyone_permission_overwrite
                .deny
                .contains(Permissions::VIEW_CHANNEL)
            {
                "invisible"
            } else if everyone_permission_overwrite
                .deny
                .contains(Permissions::CONNECT)
            {
                "locked"
            } else {
                "unlocked"
            }
        });
    let description = if current_privacy_option.eq(&privacy_option) {
        "No change has been applied."
    } else {
        let voice_channel_owner_id = interaction.voice_channel.owner_id.read().clone();
        let everyone_role_id: Id<GenericMarker> = interaction.voice_channel.guild_id.cast();
        let (mut voice_channel_owner_allow, mut voice_channel_owner_deny): (
            Permissions,
            Permissions,
        ) = (Permissions::empty(), Permissions::empty());
        let (mut everyone_allow, mut everyone_deny): (Permissions, Permissions) =
            (Permissions::empty(), Permissions::empty());

        voice_channel_permission_overwrites.retain(
            |permission_overwrite| match permission_overwrite.kind {
                ChannelPermissionOverwriteType::Member
                    if voice_channel_owner_id.is_some_and(|owner_id| {
                        owner_id.get().eq(&permission_overwrite.id.get())
                    }) =>
                {
                    voice_channel_owner_allow = permission_overwrite.allow;
                    voice_channel_owner_deny = permission_overwrite.deny;

                    false
                }
                ChannelPermissionOverwriteType::Role
                    if everyone_role_id.eq(&permission_overwrite.id) =>
                {
                    everyone_allow = permission_overwrite.allow;
                    everyone_deny = permission_overwrite.deny;

                    false
                }
                _ => {
                    !permission_overwrite.allow.is_empty() || !permission_overwrite.deny.is_empty()
                }
            },
        );

        if privacy_option.eq("invisible") {
            everyone_deny.remove(Permissions::CONNECT);
            everyone_deny.insert(Permissions::VIEW_CHANNEL);
            voice_channel_owner_allow.insert(Permissions::VIEW_CHANNEL);
        } else if privacy_option.eq("locked") {
            everyone_deny.remove(Permissions::VIEW_CHANNEL);
            everyone_deny.insert(Permissions::CONNECT);
            voice_channel_owner_allow.insert(Permissions::CONNECT);
        } else {
            everyone_deny.remove(Permissions::CONNECT | Permissions::VIEW_CHANNEL);
            voice_channel_owner_allow.remove(Permissions::CONNECT | Permissions::VIEW_CHANNEL);
        }

        voice_channel_permission_overwrites.extend(vec![ChannelPermissionOverwrite {
            allow: everyone_allow,
            deny: everyone_deny,
            id: everyone_role_id,
            kind: ChannelPermissionOverwriteType::Role,
        }]);

        if privacy_option.ne("unlocked") {
            if let Some(owner_id) = voice_channel_owner_id {
                voice_channel_permission_overwrites.extend(vec![ChannelPermissionOverwrite {
                    allow: voice_channel_owner_allow,
                    deny: voice_channel_owner_deny,
                    id: owner_id.cast(),
                    kind: ChannelPermissionOverwriteType::Member,
                }]);
            }
        }

        if context
            .client
            .update_channel(interaction.voice_channel.id)
            .permission_overwrites(&voice_channel_permission_overwrites)
            .await
            .is_err()
        {
            "I don't have permissions to update this voice channel!"
        } else if privacy_option.eq("invisible") {
            "This voice channel is now invisible."
        } else if privacy_option.eq("locked") {
            "This voice channel is now locked and visible."
        } else {
            "This voice channel is now unlocked and visible."
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
