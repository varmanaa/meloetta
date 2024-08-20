use std::sync::Arc;

use eyre::Result;
use twilight_model::gateway::payload::incoming::GuildDelete;

use crate::structs::context::Context;

pub async fn run(context: Arc<Context>, payload: GuildDelete) -> Result<()> {
    let guild_id = payload.id;

    context.cache.remove_guild(guild_id);

    if !payload.unavailable {
        context.database.remove_guild(guild_id).await?;
    }

    Ok(())
}
