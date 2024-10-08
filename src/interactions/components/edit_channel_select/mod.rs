mod allow_member_select_option;
mod claim_select_option;
mod deny_member_select_option;
mod kick_member_select_option;
mod modify_bitrate_select_option;
mod modify_name_select_option;
mod modify_privacy_select_option;
mod modify_slowmode_select_option;
mod modify_user_limit_select_option;
mod modify_video_quality_select_option;
mod remove_channel_select_option;
mod remove_member_select_option;
mod transfer_select_option;
mod view_information_select_option;

use std::{future::IntoFuture, sync::Arc};

use eyre::Result;

use crate::{
    structs::{context::Context, interaction::MessageComponentInteraction},
    utilities::{
        constants::{NON_VOICE_CHANNEL_OWNER_SELECT_OPTIONS, PANEL_MESSAGE_COMPONENTS},
        interaction::create_interaction_response_embed,
    },
};

pub async fn run(context: Arc<Context>, interaction: MessageComponentInteraction) -> Result<()> {
    if let Some(panel_message_id) = &interaction.voice_channel.panel_message_id.read().clone() {
        let context_clone = Arc::clone(&context);

        tokio::spawn(
            context_clone
                .client
                .update_message(interaction.voice_channel.id, *panel_message_id)
                .components(Some(&*PANEL_MESSAGE_COMPONENTS.clone()))
                .into_future(),
        );
    }

    let select_option = interaction.data.values.clone().into_iter().nth(0).unwrap();
    let is_user_owner = interaction
        .voice_channel
        .owner_id
        .read()
        .is_some_and(|owner_id| owner_id.eq(&interaction.user_id));
    let has_permissions = if is_user_owner {
        !NON_VOICE_CHANNEL_OWNER_SELECT_OPTIONS.contains(&select_option)
    } else {
        NON_VOICE_CHANNEL_OWNER_SELECT_OPTIONS.contains(&select_option)
    };

    if !has_permissions {
        let interaction_response =
            create_interaction_response_embed("You are not allowed to do this!".to_owned(), true);

        context
            .interaction_client()
            .create_response(interaction.id, &interaction.token, &interaction_response)
            .await?;
    } else {
        match select_option.as_str() {
            "allow-member-select-option" => {
                allow_member_select_option::run(context, interaction).await?
            }
            "claim-select-option" => claim_select_option::run(context, interaction).await?,
            "deny-member-select-option" => {
                deny_member_select_option::run(context, interaction).await?
            }
            "kick-member-select-option" => {
                kick_member_select_option::run(context, interaction).await?
            }
            "modify-bitrate-select-option" => {
                modify_bitrate_select_option::run(context, interaction).await?
            }
            "modify-name-select-option" => {
                modify_name_select_option::run(context, interaction).await?
            }
            "modify-privacy-select-option" => {
                modify_privacy_select_option::run(context, interaction).await?
            }
            "modify-slowmode-select-option" => {
                modify_slowmode_select_option::run(context, interaction).await?
            }
            "modify-user-limit-select-option" => {
                modify_user_limit_select_option::run(context, interaction).await?
            }
            "modify-video-quality-select-option" => {
                modify_video_quality_select_option::run(context, interaction).await?
            }
            "remove-channel-select-option" => {
                remove_channel_select_option::run(context, interaction).await?
            }
            "remove-member-select-option" => {
                remove_member_select_option::run(context, interaction).await?
            }
            "transfer-select-option" => transfer_select_option::run(context, interaction).await?,
            "view-information-select-option" => {
                view_information_select_option::run(context, interaction).await?
            }
            _ => {
                let interaction_response = create_interaction_response_embed(
                    format!("I don't have a select option with the name \"{select_option}\"!"),
                    true,
                );

                context
                    .interaction_client()
                    .create_response(interaction.id, &interaction.token, &interaction_response)
                    .await?;
            }
        }
    }

    Ok(())
}
