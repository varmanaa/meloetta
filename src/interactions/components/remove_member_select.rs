use std::{collections::HashSet, sync::Arc};

use eyre::Result;
use twilight_model::{channel::permission_overwrite::PermissionOverwriteType as ChannelPermissionOverwriteType, id::{marker::UserMarker, Id}};
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
        "This user may not be removed!".to_owned()
    } else if interaction
        .voice_channel
        .permission_overwrites
        .read()
        .clone()
        .iter()
        .find(|permission_overwrite| {
            permission_overwrite.id.eq(&user_id.cast())
                && permission_overwrite
                    .kind
                    .eq(&ChannelPermissionOverwriteType::Member)
        })
        .is_none()
    {
        "This user does not have permissions!".to_owned()
    } else {
        context
            .client
            .delete_channel_permission(interaction.voice_channel.id)
            .member(user_id)
            .await?;

        format!("I've removed permissions for <@{user_id}>!")
    };
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(description.to_owned())
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
