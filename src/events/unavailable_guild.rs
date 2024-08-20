use std::sync::Arc;

use twilight_model::gateway::payload::incoming::UnavailableGuild;

use crate::structs::context::Context;

use eyre::Result;

pub fn run(context: Arc<Context>, payload: UnavailableGuild) -> Result<()> {
    context.cache.insert_unavailable_guild(payload.id);

    Ok(())
}
