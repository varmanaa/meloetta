mod ban_add;
mod channel_delete;
mod channel_update;
mod guild_create;
mod guild_delete;
mod interaction_create;
mod member_remove;
mod message_delete;
mod ready;
mod unavailable_guild;
mod voice_state_update;

use std::sync::Arc;

use eyre::Result;
use twilight_gateway::Event;

use crate::structs::context::Context;

pub async fn handle_event(context: Arc<Context>, event: Event) -> Result<()> {
    match event {
        Event::BanAdd(payload) => ban_add::run(context, payload).await,
        Event::ChannelDelete(payload) => channel_delete::run(context, *payload).await,
        Event::ChannelUpdate(payload) => channel_update::run(context, *payload),
        Event::GuildCreate(payload) => guild_create::run(context, *payload).await,
        Event::GuildDelete(payload) => guild_delete::run(context, payload).await,
        Event::InteractionCreate(payload) => interaction_create::run(context, *payload).await,
        Event::MemberRemove(payload) => member_remove::run(context, payload).await,
        Event::MessageDelete(payload) => message_delete::run(context, payload).await,
        Event::Ready(payload) => ready::run(context, *payload),
        Event::UnavailableGuild(payload) => unavailable_guild::run(context, payload),
        Event::VoiceStateUpdate(payload) => voice_state_update::run(context, *payload).await,
        _ => Ok(()),
    }
}
