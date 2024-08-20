use std::sync::Arc;

use eyre::Result;
use twilight_model::gateway::payload::incoming::MessageDelete;

use crate::structs::context::Context;

pub async fn run(context: Arc<Context>, payload: MessageDelete) -> Result<()> {
    let Some(voice_channel) = context.cache.voice_channel(payload.channel_id) else {
        return Ok(());
    };
    let Some(panel_message_id) = voice_channel.panel_message_id.read().clone() else {
        return Ok(());
    };

    if panel_message_id.eq(&payload.id) {
        context
            .database
            .update_panel_message(voice_channel.id, None)
            .await?;
        context.cache.update_panel_message(voice_channel.id, None);
    }

    Ok(())
}
