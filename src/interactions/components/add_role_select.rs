use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    channel::permission_overwrite::PermissionOverwriteType as ChannelPermissionOverwriteType,
    guild::Permissions,
    http::permission_overwrite::{
        PermissionOverwrite as HttpPermissionOverwrite,
        PermissionOverwriteType as HttpPermissionOverwriteType,
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
        format!("<@&{role_id}> can't be added to this voice channel!")
    } else {
        let (mut role_allow, role_deny) = interaction
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
                (Permissions::empty(), Permissions::empty()),
                |permission_overwrite| (permission_overwrite.allow, permission_overwrite.deny),
            );
        let description_text = if role_allow.eq(&Permissions::empty()) {
            role_allow.insert(Permissions::CONNECT | Permissions::VIEW_CHANNEL);

            format!("I've added <@&{role_id}> to this voice channel!")
        } else if !role_allow.contains(Permissions::CONNECT | Permissions::VIEW_CHANNEL) {
            role_allow.insert(Permissions::CONNECT | Permissions::VIEW_CHANNEL);

            format!("I've updated permissions for <@&{role_id}> in this voice channel!")
        } else {
            format!("<@&{role_id}> already has permissions in this voice channel!")
        };

        context
            .client
            .update_channel_permission(
                interaction.voice_channel.id,
                &HttpPermissionOverwrite {
                    allow: Some(role_allow),
                    deny: Some(role_deny),
                    id: role_id,
                    kind: HttpPermissionOverwriteType::Role,
                },
            )
            .await?;

        description_text
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
