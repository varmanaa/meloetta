use std::sync::Arc;

use eyre::Result;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use crate::{
    structs::{context::Context, interaction::ApplicationCommandInteraction},
    utilities::interaction::create_deferred_interaction_response,
};

pub async fn run(context: Arc<Context>, interaction: ApplicationCommandInteraction) -> Result<()> {
    let interaction_response = create_deferred_interaction_response(true);

    context
        .interaction_client()
        .create_response(interaction.id, &interaction.token, &interaction_response)
        .await?;

    let categories_text = if interaction.guild.category_channel_ids.read().is_empty() {
        "No voice channel categories have been created.".to_owned()
    } else {
        interaction
            .guild
            .category_channel_ids
            .read()
            .iter()
            .map(|channel_id| {
                context
                    .cache
                    .category_channel(*channel_id)
                    .map_or(format!("- {channel_id} **(no longer exists)**"), |_| {
                        format!("- <#{channel_id}>")
                    })
            })
            .collect::<Vec<String>>()
            .join("\n")
    };
    let permanence_text = if interaction.guild.permanence.read().clone() {
        "Voice channels **will not be deleted** when empty.".to_owned()
    } else {
        "Voice channels **will be deleted** when empty.".to_owned()
    };
    let privacy_text = match interaction.guild.privacy.read().clone().as_str() {
        "invisible" => "Voice channels are **invisible** by default.",
        "locked" => "Voice channels are **locked and visible** by default.",
        _ => "Voice channels are **not locked and visible** by default.",
    };
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .field(EmbedFieldBuilder::new("Categories", categories_text).build())
        .field(EmbedFieldBuilder::new("Permanence", permanence_text).build())
        .field(EmbedFieldBuilder::new("Privacy", privacy_text).build())
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
