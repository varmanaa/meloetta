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
    let options = [("Auto", "1"), ("720p", "2")]
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
            custom_id: "modify-video-quality-select".to_owned(),
            default_values: None,
            disabled: false,
            kind: SelectMenuType::Text,
            max_values: Some(1),
            min_values: Some(1),
            options: Some(options),
            placeholder: Some("Modify video quality...".to_owned()),
        })],
    })];
    let interaction_response = create_interaction_response_select(components, true);

    context
        .interaction_client()
        .create_response(interaction.id, &interaction.token, &interaction_response)
        .await?;

    Ok(())
}
