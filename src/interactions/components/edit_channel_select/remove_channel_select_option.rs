use std::sync::Arc;

use eyre::Result;

use crate::structs::{context::Context, interaction::MessageComponentInteraction};

pub async fn run(context: Arc<Context>, interaction: MessageComponentInteraction) -> Result<()> {
    context
        .client
        .delete_channel(interaction.voice_channel.id)
        .await?;

    Ok(())
}
