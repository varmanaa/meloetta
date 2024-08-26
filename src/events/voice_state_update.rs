use std::sync::Arc;

use twilight_model::{
    channel::{
        permission_overwrite::{
            PermissionOverwrite as ChannelPermissionOverwrite,
            PermissionOverwriteType as ChannelPermissionOverwriteType,
        },
        ChannelType,
    },
    gateway::payload::incoming::VoiceStateUpdate,
    guild::Permissions,
    id::{marker::GenericMarker, Id},
};

use crate::{
    structs::context::Context,
    utilities::constants::{PANEL_MESSAGE_COMPONENTS, PANEL_MESSAGE_EMBED},
};

use eyre::Result;

pub async fn run(context: Arc<Context>, payload: VoiceStateUpdate) -> Result<()> {
    let Some(guild_id) = payload.guild_id else {
        return Ok(());
    };
    let Some(guild) = context.cache.guild(guild_id) else {
        return Ok(());
    };
    let user_id = payload.0.user_id;

    if let Some(old_channel_id) = context.cache.voice_state(guild_id, user_id) {
        context.cache.remove_voice_state(guild_id, user_id);

        if guild.permanence.read().clone() {
            return Ok(());
        };

        if let Some(old_channel) = context.cache.voice_channel(*old_channel_id) {
            if old_channel.connected_user_ids.read().is_empty() {
                _ = context.client.delete_channel(*old_channel_id).await;
            }
        }
    }
    if let Some(new_channel_id) = payload.0.channel_id {
        context
            .cache
            .insert_voice_state(guild_id, new_channel_id, user_id);

        if context
            .cache
            .voice_channel_owner(guild_id, user_id)
            .is_some()
        {
            return Ok(());
        }

        let category_channel = guild.category_channel_ids.read().iter().find_map(|id| {
            let Some(category_channel) = context.cache.category_channel(*id) else {
                return None;
            };
            let Some(join_channel_id) = category_channel.join_channel_id.read().clone() else {
                return None;
            };

            join_channel_id
                .eq(&new_channel_id)
                .then_some(category_channel)
        });
        let Some(category_channel) = category_channel else {
            return Ok(());
        };

        let Some(member) = payload.0.member else {
            return Ok(());
        };
        let channel_name = if member.user.name.ends_with("s") {
            format!("{}' voice", member.user.name)
        } else {
            format!("{}'s voice", member.user.name)
        };

        let mut permission_overwrites = category_channel.permission_overwrites.read().clone();
        let everyone_role_id: Id<GenericMarker> = guild_id.cast();
        let (mut voice_channel_owner_allow, mut voice_channel_owner_deny): (
            Permissions,
            Permissions,
        ) = (Permissions::empty(), Permissions::empty());
        let (mut everyone_allow, mut everyone_deny): (Permissions, Permissions) =
            (Permissions::empty(), Permissions::empty());
        let privacy_option = guild.privacy.read().clone();

        permission_overwrites.retain(|permission_overwrite| match permission_overwrite.kind {
            ChannelPermissionOverwriteType::Member
                if user_id.get().eq(&permission_overwrite.id.get()) =>
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
            ChannelPermissionOverwriteType::Role
                if guild.bot_role_id.cast().eq(&permission_overwrite.id) =>
            {
                false
            }
            _ => !permission_overwrite.allow.is_empty() || !permission_overwrite.deny.is_empty(),
        });

        if privacy_option.eq("invisible") {
            everyone_deny.insert(Permissions::VIEW_CHANNEL);
            voice_channel_owner_allow.insert(Permissions::VIEW_CHANNEL);
        } else if privacy_option.eq("locked") {
            everyone_deny.insert(Permissions::CONNECT);
            voice_channel_owner_allow.insert(Permissions::CONNECT);
        }

        permission_overwrites.extend(vec![
            ChannelPermissionOverwrite {
                allow: Permissions::CONNECT | Permissions::VIEW_CHANNEL,
                deny: Permissions::empty(),
                id: guild.bot_role_id.cast(),
                kind: ChannelPermissionOverwriteType::Role,
            },
            ChannelPermissionOverwrite {
                allow: everyone_allow,
                deny: everyone_deny,
                id: everyone_role_id,
                kind: ChannelPermissionOverwriteType::Role,
            },
            ChannelPermissionOverwrite {
                allow: voice_channel_owner_allow,
                deny: voice_channel_owner_deny,
                id: user_id.cast(),
                kind: ChannelPermissionOverwriteType::Member,
            },
        ]);

        if let Ok(created_voice_channel_response) = context
            .client
            .create_guild_channel(guild_id, &channel_name)
            .kind(ChannelType::GuildVoice)
            .parent_id(category_channel.id)
            .permission_overwrites(&permission_overwrites)
            .await
        {
            let created_voice_channel = created_voice_channel_response.model().await?;
            let panel_message_id = Some(
                context
                    .client
                    .create_message(created_voice_channel.id)
                    .components(&*PANEL_MESSAGE_COMPONENTS)
                    .embeds(&[PANEL_MESSAGE_EMBED.clone()])
                    .await?
                    .model()
                    .await?
                    .id,
            );
            context
                .client
                .update_guild_member(guild_id, user_id)
                .channel_id(Some(created_voice_channel.id))
                .await?;
            context
                .database
                .insert_voice_channel(
                    created_voice_channel.id,
                    guild_id,
                    category_channel.id,
                    user_id,
                )
                .await?;
            context.cache.insert_voice_channel(
                None,
                guild_id,
                created_voice_channel.id,
                Some(user_id),
                panel_message_id,
                category_channel.id,
                created_voice_channel
                    .permission_overwrites
                    .unwrap_or_default(),
            );
        }
    }

    Ok(())
}
