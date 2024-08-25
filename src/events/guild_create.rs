use std::{collections::HashMap, sync::Arc};

use eyre::{Ok, Result};
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, ChannelType},
    gateway::payload::incoming::GuildCreate,
    id::{
        marker::{ChannelMarker, UserMarker},
        Id,
    },
};

use crate::structs::context::Context;

pub async fn run(context: Arc<Context>, payload: GuildCreate) -> Result<()> {
    let guild_id = payload.0.id;
    let Some(bot_role) = payload.0.roles.into_iter().find(|role| {
        role.tags.as_ref().is_some_and(|tags| {
            tags.bot_id
                .is_some_and(|bot_id| bot_id.eq(&context.application_id.cast()))
        })
    }) else {
        context.client.leave_guild(guild_id).await?;

        return Ok(());
    };

    context.database.insert_guild(guild_id).await?;

    let database_guild = context.database.guild(guild_id).await?.unwrap();

    context.cache.insert_guild(
        guild_id,
        bot_role.id,
        database_guild.permanence,
        database_guild.privacy,
    );

    let mut category_channel_permission_overwrites_map: HashMap<
        Id<ChannelMarker>,
        Vec<PermissionOverwrite>,
    > = HashMap::new();
    let mut voice_channel_permission_overwrites_map: HashMap<
        Id<ChannelMarker>,
        Vec<PermissionOverwrite>,
    > = HashMap::new();
    let mut voice_channel_and_parent_ids: Vec<(Id<ChannelMarker>, Id<ChannelMarker>)> = Vec::new();
    let category_and_voice_channel_ids = payload
        .0
        .channels
        .iter()
        .filter_map(|channel| {
            let channel_id = channel.id;

            match channel.kind {
                ChannelType::GuildCategory => {
                    category_channel_permission_overwrites_map.insert(
                        channel_id,
                        channel.permission_overwrites.clone().unwrap_or_default(),
                    );

                    Some(channel_id)
                }
                ChannelType::GuildVoice => {
                    if let Some(parent_id) = channel.parent_id {
                        voice_channel_permission_overwrites_map.insert(
                            channel_id,
                            channel.permission_overwrites.clone().unwrap_or_default(),
                        );
                        voice_channel_and_parent_ids.push((channel_id, parent_id));
                    }

                    Some(channel_id)
                }
                _ => None,
            }
        })
        .collect::<Vec<Id<ChannelMarker>>>();

    if !category_and_voice_channel_ids.is_empty() {
        context
            .database
            .remove_channels(guild_id, category_and_voice_channel_ids)
            .await?;
    }
    if !voice_channel_and_parent_ids.is_empty() {
        context
            .database
            .update_channels(guild_id, voice_channel_and_parent_ids)
            .await?;
    }

    let database_guild_category_channels =
        context.database.guild_category_channels(guild_id).await?;
    let database_guild_voice_channels = context.database.guild_voice_channels(guild_id).await?;
    let database_guild_voice_channel_map: HashMap<Id<ChannelMarker>, Vec<Id<ChannelMarker>>> =
        database_guild_voice_channels.iter().fold(
            HashMap::new(),
            |mut acc, database_guild_voice_channel| {
                let parent_id = database_guild_voice_channel.parent_id;
                let channel_id = database_guild_voice_channel.id;

                if !acc.contains_key(&parent_id) {
                    acc.insert(parent_id, vec![channel_id]);
                } else if let Some(category_voice_channels) = acc.get_mut(&parent_id) {
                    category_voice_channels.push(channel_id);
                };

                acc
            },
        );
    let voice_state_map: HashMap<Id<ChannelMarker>, Vec<Id<UserMarker>>> = payload
        .0
        .voice_states
        .iter()
        .fold(HashMap::new(), |mut acc, voice_state| {
            if let Some(channel_id) = voice_state.channel_id {
                let user_id = voice_state.user_id;

                if !acc.contains_key(&channel_id) {
                    acc.insert(channel_id, vec![user_id]);
                } else if let Some(user_ids_in_channel) = acc.get_mut(&channel_id) {
                    user_ids_in_channel.push(user_id);
                };
            }

            acc
        });

    for database_guild_category_channel in database_guild_category_channels {
        let channel_id = database_guild_category_channel.id;
        let permission_overwrites = category_channel_permission_overwrites_map
            .get(&channel_id)
            .cloned()
            .unwrap_or_default();
        let voice_channel_ids = database_guild_voice_channel_map
            .get(&channel_id)
            .cloned()
            .unwrap_or_default();

        context.cache.insert_category_channel(
            guild_id,
            channel_id,
            database_guild_category_channel.join_channel_id,
            permission_overwrites,
            voice_channel_ids,
        );
    }

    for database_guild_voice_channel in database_guild_voice_channels {
        let channel_id = database_guild_voice_channel.id;
        let connected_user_ids = voice_state_map
            .get(&channel_id)
            .cloned()
            .unwrap_or_default();
        let permission_overwrites = voice_channel_permission_overwrites_map
            .get(&channel_id)
            .cloned()
            .unwrap_or_default();

        context.cache.insert_voice_channel(
            connected_user_ids,
            guild_id,
            channel_id,
            database_guild_voice_channel.owner_id,
            database_guild_voice_channel.panel_message_id,
            database_guild_voice_channel.parent_id,
            permission_overwrites,
        );
    }

    Ok(())
}
