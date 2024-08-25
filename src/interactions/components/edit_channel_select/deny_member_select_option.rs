use std::sync::Arc;

use eyre::Result;
use twilight_model::channel::message::{
    component::{ActionRow, SelectMenu, SelectMenuType},
    Component,
};

use crate::{
    structs::{context::Context, interaction::MessageComponentInteraction},
    utilities::interaction::create_interaction_response_select,
};

pub async fn run(context: Arc<Context>, interaction: MessageComponentInteraction) -> Result<()> {
    let components = vec![Component::ActionRow(ActionRow {
        components: vec![Component::SelectMenu(SelectMenu {
            channel_types: None,
            custom_id: "deny-member-select".to_owned(),
            default_values: None,
            disabled: false,
            kind: SelectMenuType::User,
            max_values: Some(1),
            min_values: Some(1),
            options: None,
            placeholder: Some("Select a member to deny permissions for...".to_owned()),
            
        })],
    })];
    let interaction_response = create_interaction_response_select(components, true);

    context
        .interaction_client()
        .create_response(interaction.id, &interaction.token, &interaction_response)
        .await?;

    Ok(())
}
