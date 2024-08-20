use std::sync::Arc;

use eyre::Result;
use twilight_model::{channel::ChannelType, gateway::payload::incoming::ChannelUpdate};

use crate::structs::context::Context;

pub fn run(context: Arc<Context>, payload: ChannelUpdate) -> Result<()> {
    let channel_id = payload.0.id;

    match payload.0.kind {
        ChannelType::GuildCategory => {
            if let Some(category_channel) = context.cache.category_channel(channel_id) {
                context.cache.insert_category_channel(
                    category_channel.guild_id,
                    category_channel.id,
                    category_channel.join_channel_id.read().clone(),
                    payload.0.permission_overwrites.unwrap_or_default(),
                    category_channel.voice_channel_ids.read().clone(),
                )
            }
        }
        ChannelType::GuildVoice => {
            if let Some(voice_channel) = context.cache.voice_channel(channel_id) {
                context.cache.insert_voice_channel(
                    voice_channel.connected_user_ids.read().clone(),
                    voice_channel.guild_id,
                    voice_channel.id,
                    voice_channel.owner_id.read().clone(),
                    voice_channel.panel_message_id.read().clone(),
                    voice_channel.parent_id,
                    payload.0.permission_overwrites.unwrap_or_default(),
                )
            }
        }
        _ => {}
    }

    Ok(())
}
