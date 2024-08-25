use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    application::interaction::InteractionData, gateway::payload::incoming::InteractionCreate,
};

use crate::{
    interactions::*,
    structs::{
        context::Context,
        interaction::{
            ApplicationCommandInteraction, MessageComponentInteraction, ModalSubmitInteraction,
        },
    },
    utilities::interaction::{check_interaction, create_interaction_response_embed},
};

async fn handle_application_command(
    context: Arc<Context>,
    interaction: ApplicationCommandInteraction,
) -> Result<()> {
    let application_command_name = interaction.data.name.as_str();

    match application_command_name {
        "create" => create::run(context, interaction).await?,
        "settings" => settings::run(context, interaction).await?,
        _ => {
            let interaction_response = create_interaction_response_embed(
                format!("I don't have a command with the name \"{application_command_name}\"!"),
                true,
            );

            context
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &interaction_response)
                .await?;
        }
    }

    Ok(())
}

pub async fn run(context: Arc<Context>, payload: InteractionCreate) -> Result<()> {
    let interaction = payload.0;

    let (guild, voice_channel) = match check_interaction(&context, &interaction) {
        Err(report) => {
            let interaction_response = create_interaction_response_embed(report.to_string(), true);

            context
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &interaction_response)
                .await?;

            return Ok(());
        }
        Ok(data) => data,
    };

    match (interaction.data, guild, voice_channel) {
        (Some(InteractionData::ApplicationCommand(data)), Some(guild), None) => {
            let interaction = ApplicationCommandInteraction {
                data,
                guild,
                id: interaction.id,
                token: interaction.token,
            };

            handle_application_command(context, interaction).await?;
        }
        (Some(InteractionData::MessageComponent(data)), None, Some(voice_channel)) => {
            let interaction = MessageComponentInteraction {
                data,
                id: interaction.id,
                token: interaction.token,
                user_id: interaction.member.unwrap().user.unwrap().id,
                voice_channel,
            };

            handle_message_component(context, interaction).await?;
        }
        (Some(InteractionData::ModalSubmit(data)), None, Some(voice_channel)) => {
            let interaction = ModalSubmitInteraction {
                data,
                id: interaction.id,
                token: interaction.token,
                voice_channel,
            };

            handle_modal_submit(context, interaction).await?;
        }
        _ => {
            let interaction_response = create_interaction_response_embed(
                "I don't recognize this interaction.".to_owned(),
                true,
            );

            context
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &interaction_response)
                .await?;
        }
    }

    Ok(())
}

async fn handle_message_component(
    context: Arc<Context>,
    interaction: MessageComponentInteraction,
) -> Result<()> {
    let message_component_name = interaction.data.custom_id.as_str();

    match message_component_name {
        "allow-member-select" => allow_member_select::run(context, interaction).await?,
        "deny-member-select" => deny_member_select::run(context, interaction).await?,
        "edit-channel-select" => edit_channel_select::run(context, interaction).await?,
        "kick-member-select" => kick_member_select::run(context, interaction).await?,
        "modify-privacy-select" => modify_privacy_select::run(context, interaction).await?,
        "modify-slowmode-select" => modify_slowmode_select::run(context, interaction).await?,
        "modify-video-quality-select" => {
            modify_video_quality_select::run(context, interaction).await?
        }
        "remove-member-select" => remove_member_select::run(context, interaction).await?,
        "transfer-select" => transfer_select::run(context, interaction).await?,
        _ => {
            let interaction_response = create_interaction_response_embed(
                format!("I don't have a component with the name \"{message_component_name}\"!"),
                true,
            );

            context
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &interaction_response)
                .await?;
        }
    }

    Ok(())
}

async fn handle_modal_submit(
    context: Arc<Context>,
    interaction: ModalSubmitInteraction,
) -> Result<()> {
    let modal_submit_name = interaction.data.custom_id.as_str();

    match modal_submit_name {
        "modify-bitrate-modal" => modify_bitrate_modal::run(context, interaction).await?,
        "modify-name-modal" => modify_name_modal::run(context, interaction).await?,
        "modify-user-limit-modal" => modify_user_limit_modal::run(context, interaction).await?,
        _ => {
            let interaction_response = create_interaction_response_embed(
                format!("I don't have a modal with the name \"{modal_submit_name}\"!"),
                true,
            );

            context
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &interaction_response)
                .await?;
        }
    }

    Ok(())
}
