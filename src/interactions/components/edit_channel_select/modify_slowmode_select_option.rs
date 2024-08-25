use std::sync::Arc;

use eyre::Result;
use twilight_model::channel::message::{
    component::{ActionRow, SelectMenu, SelectMenuOption, SelectMenuType},
    Component,
};

use crate::{
    structs::{context::Context, interaction::MessageComponentInteraction},
    utilities::interaction::create_interaction_response_select,
};

pub async fn run(context: Arc<Context>, interaction: MessageComponentInteraction) -> Result<()> {
    let options = [
        ("Off", "0"),
        ("5s", "5"),
        ("10s", "10"),
        ("15s", "15"),
        ("30s", "30"),
        ("1m", "60"),
        ("2m", "120"),
        ("5m", "300"),
        ("10m", "600"),
        ("15m", "900"),
        ("30m", "1800"),
        ("1h", "3600"),
        ("2h", "7200"),
        ("6h", "21600"),
    ]
    .into_iter()
    .map(|(label, value)| SelectMenuOption {
        default: false,
        description: None,
        emoji: None,
        label: label.to_owned(),
        value: value.to_owned(),
    })
    .collect::<Vec<SelectMenuOption>>();
    let components = vec![Component::ActionRow(ActionRow {
        components: vec![Component::SelectMenu(SelectMenu {
            channel_types: None,
            custom_id: "modify-slowmode-select".to_owned(),
            default_values: None,
            disabled: false,
            kind: SelectMenuType::Text,
            max_values: Some(1),
            min_values: Some(1),
            options: Some(options),
            placeholder: Some("Modify slowmode...".to_owned()),
        })],
    })];
    let interaction_response = create_interaction_response_select(components, true);

    context
        .interaction_client()
        .create_response(interaction.id, &interaction.token, &interaction_response)
        .await?;

    Ok(())
}
