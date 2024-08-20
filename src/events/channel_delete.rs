use std::sync::Arc;

use eyre::Result;
use twilight_model::{channel::ChannelType, gateway::payload::incoming::ChannelDelete};

use crate::structs::context::Context;

pub async fn run(context: Arc<Context>, payload: ChannelDelete) -> Result<()> {
    let channel_id = payload.0.id;

    match payload.0.kind {
        ChannelType::GuildCategory => {
            context.database.remove_category_channel(channel_id).await?;
            context.cache.remove_category_channel(channel_id);
        }
        ChannelType::GuildVoice => {
            let Some(guild_id) = payload.0.guild_id else {
                return Ok(());
            };
            let Some(guild) = context.cache.guild(guild_id) else {
                return Ok(());
            };
            let guild_category_channel_ids = guild.category_channel_ids.read().clone();
            let category_channel =
                guild_category_channel_ids
                    .iter()
                    .find_map(|category_channel_id| {
                        let Some(category_channel) =
                            context.cache.category_channel(*category_channel_id)
                        else {
                            return None;
                        };
                        let Some(join_channel_id) = category_channel.join_channel_id.read().clone()
                        else {
                            return None;
                        };

                        join_channel_id.eq(&channel_id).then_some(category_channel)
                    });

            if let Some(category_channel) = category_channel {
                context
                    .database
                    .update_join_channel(category_channel.id, None)
                    .await?;
                context
                    .cache
                    .update_join_channel(category_channel.id, None);
            } else {
                context.database.remove_voice_channel(channel_id).await?;
                context.cache.remove_voice_channel(channel_id);
            }
        }
        _ => {}
    }

    Ok(())
}
