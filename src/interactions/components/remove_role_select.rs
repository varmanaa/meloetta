use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    channel::permission_overwrite::PermissionOverwriteType as ChannelPermissionOverwriteType,
    guild::Permissions,
    http::permission_overwrite::{
        PermissionOverwrite, PermissionOverwriteType as HttpPermissionOverwriteType,
    },
    id::{marker::GenericMarker, Id},
};
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

    let is_managed_role = interaction
        .data
        .resolved
        .is_some_and(|data| data.roles.values().nth(0).is_some_and(|role| role.managed));
    let role_id: Id<GenericMarker> = interaction.data.values.iter().nth(0).unwrap().parse()?;
    let description = if is_managed_role {
        format!("<@&{role_id}> can't be removed from this voice channel!")
    } else {
        let (mut role_allow, role_deny, role_permission_overwrite_exists) = interaction
            .voice_channel
            .permission_overwrites
            .read()
            .iter()
            .find(|permission_overwrite| {
                permission_overwrite.id.eq(&role_id)
                    && permission_overwrite
                        .kind
                        .eq(&ChannelPermissionOverwriteType::Role)
            })
            .map_or(
                (Permissions::empty(), Permissions::empty(), false),
                |permission_overwrite| {
                    (permission_overwrite.allow, permission_overwrite.deny, true)
                },
            );

        if !role_permission_overwrite_exists {
            format!("<@&{role_id}> doesn't have permissions in this voice channel!")
        } else {
            role_allow.remove(Permissions::CONNECT | Permissions::VIEW_CHANNEL);

            if role_allow.is_empty() && role_deny.is_empty() {
                context
                    .client
                    .delete_channel_permission(interaction.voice_channel.id)
                    .role(role_id.cast())
                    .await?;
            } else {
                context
                    .client
                    .update_channel_permission(
                        interaction.voice_channel.id,
                        &PermissionOverwrite {
                            allow: Some(role_allow),
                            deny: Some(role_deny),
                            id: role_id,
                            kind: HttpPermissionOverwriteType::Role,
                        },
                    )
                    .await?;
            }

            format!("I've removed permissions for <@&{role_id}> in this voice channel!")
        }
    };
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(description)
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
