use std::{collections::HashSet, iter, sync::Arc};

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

    let user_id: Id<GenericMarker> = interaction.data.values.iter().nth(0).unwrap().parse()?;
    let mut permanent_user_ids: HashSet<Id<GenericMarker>> =
        HashSet::from_iter(iter::once(context.application_id.cast()));

    if let Some(owner_id) = interaction.voice_channel.owner_id.read().clone() {
        permanent_user_ids.insert(owner_id.cast());
    };

    let description = if permanent_user_ids.contains(&user_id) {
        format!("<@{user_id}> can't be removed from this voice channel!")
    } else {
        let (mut member_allow, member_deny, member_permission_overwrite_exists) = interaction
            .voice_channel
            .permission_overwrites
            .read()
            .iter()
            .find(|permission_overwrite| {
                permission_overwrite.id.eq(&user_id)
                    && permission_overwrite
                        .kind
                        .eq(&ChannelPermissionOverwriteType::Member)
            })
            .map_or(
                (Permissions::empty(), Permissions::empty(), false),
                |permission_overwrite| {
                    (permission_overwrite.allow, permission_overwrite.deny, true)
                },
            );

        if !member_permission_overwrite_exists {
            format!("<@{user_id}> doesn't have permissions in this voice channel!")
        } else {
            member_allow.remove(Permissions::CONNECT | Permissions::VIEW_CHANNEL);

            if member_allow.is_empty() && member_deny.is_empty() {
                context
                    .client
                    .delete_channel_permission(interaction.voice_channel.id)
                    .member(user_id.cast())
                    .await?;
            } else {
                context
                    .client
                    .update_channel_permission(
                        interaction.voice_channel.id,
                        &PermissionOverwrite {
                            allow: Some(member_allow),
                            deny: Some(member_deny),
                            id: user_id,
                            kind: HttpPermissionOverwriteType::Member,
                        },
                    )
                    .await?;
            }

            format!("I've removed permissions for <@{user_id}> in this voice channel!")
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
