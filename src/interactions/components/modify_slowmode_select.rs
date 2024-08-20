use std::{collections::HashMap, sync::Arc};

use eyre::Result;
use twilight_util::builder::embed::EmbedBuilder;

use crate::{
    structs::{context::Context, interaction::MessageComponentInteraction},
    utilities::interaction::create_deferred_interaction_response,
};

pub async fn run(context: Arc<Context>, interaction: MessageComponentInteraction) -> Result<()> {
    let interaction_response = create_deferred_interaction_response(true);

    context
        .interaction_client()
        .create_response(interaction.id, &interaction.token, &interaction_response)
        .await?;

    let slowmode_option = interaction.data.values.clone().into_iter().nth(0).unwrap();
    let amount = slowmode_option.parse::<u16>()?;

    context
        .client
        .update_channel(interaction.voice_channel.id)
        .rate_limit_per_user(amount)
        .await?;

    let options: HashMap<u16, &str> = HashMap::from_iter(vec![
        (0, "Off"),
        (5, "5s"),
        (10, "10s"),
        (15, "15s"),
        (30, "30s"),
        (60, "1m"),
        (120, "2m"),
        (300, "5m"),
        (600, "10m"),
        (900, "15m"),
        (1800, "30m"),
        (3600, "1h"),
        (7200, "2h"),
        (21600, "6h"),
    ]);
    let description = format!(
        "I've modified the slowmode to {} in this voice channel!",
        options.get(&amount).cloned().unwrap().to_owned()
    );
    let embed = EmbedBuilder::new().color(0xF8F8FF).description(description).build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
