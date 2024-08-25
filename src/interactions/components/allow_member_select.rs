use std::{collections::HashSet, sync::Arc};

use eyre::Result;

use twilight_model::{
    channel::permission_overwrite::PermissionOverwriteType as ChannelPermissionOverwriteType,
    guild::Permissions,
    http::permission_overwrite::{
        PermissionOverwrite as HttpPermissionOverwrite,
        PermissionOverwriteType as HttpPermissionOverwriteType,
    },
    id::{marker::UserMarker, Id},
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

    let user_id = interaction
        .data
        .resolved
        .unwrap()
        .users
        .values()
        .nth(0)
        .unwrap()
        .id;
    let protected_user_ids: HashSet<Id<UserMarker>> = HashSet::from_iter(vec![
        interaction.voice_channel.owner_id.read().clone().unwrap(),
        context.application_id.cast(),
    ]);
    let description = if protected_user_ids.contains(&user_id) {
        "This user may not be added!".to_owned()
    } else {
        let (mut member_allow, member_deny) = interaction
            .voice_channel
            .permission_overwrites
            .read()
            .clone()
            .into_iter()
            .find(|permission_overwrite| {
                permission_overwrite.id.eq(&user_id.cast())
                    && permission_overwrite
                        .kind
                        .eq(&ChannelPermissionOverwriteType::Member)
            })
            .map_or(
                (Permissions::empty(), Permissions::empty()),
                |member_permission_overwrite| {
                    (
                        member_permission_overwrite.allow,
                        member_permission_overwrite.deny,
                    )
                },
            );

        member_allow.insert(Permissions::CONNECT | Permissions::VIEW_CHANNEL);

        if context
            .client
            .update_channel_permission(
                interaction.voice_channel.id,
                &HttpPermissionOverwrite {
                    allow: Some(member_allow),
                    deny: Some(member_deny),
                    id: user_id.cast(),
                    kind: HttpPermissionOverwriteType::Member,
                },
            )
            .await
            .is_err()
        {
            "I don't have permissions to add users to this voice channel!".to_owned()
        } else {
            format!("I've added permissions for <@{user_id}>!")
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
