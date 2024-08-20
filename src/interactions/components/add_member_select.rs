use std::{collections::HashSet, iter, sync::Arc};

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

    let user_id: Id<GenericMarker> = interaction.data.values.iter().nth(0).unwrap().parse()?;
    let mut permanent_user_ids: HashSet<Id<GenericMarker>> =
        HashSet::from_iter(iter::once(context.application_id.cast()));

    if let Some(owner_id) = interaction.voice_channel.owner_id.read().clone() {
        permanent_user_ids.insert(owner_id.cast());
    };

    let description = if permanent_user_ids.contains(&user_id) {
        format!("<@{user_id}> can't be added to this voice channel!")
    } else {
        let (mut member_allow, member_deny) = interaction
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
                (Permissions::empty(), Permissions::empty()),
                |permission_overwrite| (permission_overwrite.allow, permission_overwrite.deny),
            );
        let description_text = if member_allow.eq(&Permissions::empty()) {
            member_allow.insert(Permissions::CONNECT | Permissions::VIEW_CHANNEL);

            format!("I've added <@{user_id}> to this voice channel!")
        } else if !member_allow.contains(Permissions::CONNECT | Permissions::VIEW_CHANNEL) {
            member_allow.insert(Permissions::CONNECT | Permissions::VIEW_CHANNEL);

            format!("I've updated permissions for <@{user_id}> in this voice channel!")
        } else {
            format!("<@{user_id}> already has permissions in this voice channel!")
        };
        
        context
            .client
            .update_channel_permission(
                interaction.voice_channel.id,
                &HttpPermissionOverwrite {
                    allow: Some(member_allow),
                    deny: Some(member_deny),
                    id: user_id,
                    kind: HttpPermissionOverwriteType::Member,
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
