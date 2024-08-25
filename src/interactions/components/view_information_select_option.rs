use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    channel::permission_overwrite::PermissionOverwriteType as ChannelPermissionOverwriteType,
    guild::Permissions,
    id::{marker::GenericMarker, Id},
};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

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

    let mut allowed_member_ids = interaction
        .voice_channel
        .permission_overwrites
        .read()
        .clone()
        .into_iter()
        .filter_map(|permission_overwrite| {
            permission_overwrite
                .kind
                .eq(&ChannelPermissionOverwriteType::Member)
                .then(|| permission_overwrite.id)
        })
        .collect::<Vec<Id<GenericMarker>>>();
    let members_text = if allowed_member_ids.is_empty() {
        "No members have been added to this voice channel.".to_owned()
    } else {
        let mut text = allowed_member_ids
            .iter()
            .map(|id| format!("- <@{id}>"))
            .collect::<Vec<String>>()
            .join("\n");

        if allowed_member_ids.len() > 5 {
            let remaining_members = allowed_member_ids.split_off(5);

            text.push_str(&format!("\n +{} more", remaining_members.len()));
        }

        text
    };
    let everyone_deny = interaction
        .voice_channel
        .permission_overwrites
        .read()
        .clone()
        .into_iter()
        .find(|permission_overwrite| {
            permission_overwrite
                .id
                .eq(&interaction.voice_channel.guild_id.cast())
                && permission_overwrite
                    .kind
                    .eq(&ChannelPermissionOverwriteType::Role)
        })
        .map_or(Permissions::empty(), |permission_overwrite| {
            permission_overwrite.deny
        });
    let privacy_text = if everyone_deny.contains(Permissions::VIEW_CHANNEL) {
        "This voice channel is invisible."
    } else if everyone_deny.contains(Permissions::CONNECT) {
        "This voice channel is locked and visible."
    } else {
        "This voice channel is unlocked and visible."
    };
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .field(EmbedFieldBuilder::new("Allowed member(s)", members_text).build())
        .field(EmbedFieldBuilder::new("Privacy", privacy_text).build())
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
