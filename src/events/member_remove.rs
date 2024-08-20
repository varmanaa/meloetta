use std::sync::Arc;

use eyre::Result;
use twilight_model::gateway::payload::incoming::MemberRemove;

use crate::structs::context::Context;

pub async fn run(context: Arc<Context>, payload: MemberRemove) -> Result<()> {
    let user_id = payload.user.id;

    if let Some(voice_channel_id) = context.cache.voice_channel_owner(payload.guild_id, user_id) {
        context
            .database
            .update_voice_channel_owner(*voice_channel_id, None)
            .await?;
        context
            .cache
            .update_voice_channel_owner(*voice_channel_id, None);
    }

    Ok(())
}
