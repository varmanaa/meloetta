use std::sync::Arc;

use twilight_model::{
    application::interaction::{
        application_command::CommandData, message_component::MessageComponentInteractionData,
        modal::ModalInteractionData,
    },
    id::{
        marker::{InteractionMarker, UserMarker},
        Id,
    },
};

use super::cache::{CachedGuild, CachedVoiceChannel};

pub struct ApplicationCommandInteraction {
    pub data: Box<CommandData>,
    pub guild: Arc<CachedGuild>,
    pub id: Id<InteractionMarker>,
    pub token: String,
}

#[derive(Clone)]
pub struct MessageComponentInteraction {
    pub data: Box<MessageComponentInteractionData>,
    pub id: Id<InteractionMarker>,
    pub token: String,
    pub user_id: Id<UserMarker>,
    pub voice_channel: Arc<CachedVoiceChannel>,
}

pub struct ModalSubmitInteraction {
    pub data: ModalInteractionData,
    pub id: Id<InteractionMarker>,
    pub token: String,
    pub voice_channel: Arc<CachedVoiceChannel>,
}
