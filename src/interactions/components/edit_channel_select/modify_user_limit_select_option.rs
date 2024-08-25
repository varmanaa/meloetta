use std::sync::Arc;

use eyre::Result;
use twilight_model::channel::message::{
    component::{ActionRow, TextInput, TextInputStyle},
    Component,
};

use crate::{
    structs::{context::Context, interaction::MessageComponentInteraction},
    utilities::interaction::create_interaction_response_modal,
};

pub async fn run(context: Arc<Context>, interaction: MessageComponentInteraction) -> Result<()> {
    let components = vec![Component::ActionRow(ActionRow {
        components: vec![Component::TextInput(TextInput {
            custom_id: "modify-user-limit-text-input".to_owned(),
            required: Some(true),
            placeholder: Some("Enter a new user limit (Set to 0 for no limit!)...".to_owned()),
            label: "User limit".to_owned(),
            max_length: Some(2),
            min_length: Some(1),
            style: TextInputStyle::Short,
            value: None,
        })],
    })];
    let interaction_response = create_interaction_response_modal(
        "modify-user-limit-modal".to_owned(),
        components,
        "Modify user limit".to_owned(),
    );

    context
        .interaction_client()
        .create_response(interaction.id, &interaction.token, &interaction_response)
        .await?;
    
    Ok(())
}
