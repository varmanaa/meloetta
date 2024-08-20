use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    channel::permission_overwrite::PermissionOverwriteType,
    guild::Permissions,
    id::{
        marker::{RoleMarker, UserMarker},
        Id,
    },
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

    let mut added_member_ids: Vec<Id<UserMarker>> = Vec::new();
    let mut added_role_ids: Vec<Id<RoleMarker>> = Vec::new();
    let mut is_locked: bool = false;

    for permission_overwrite in interaction
        .voice_channel
        .permission_overwrites
        .read()
        .clone()
    {
        match permission_overwrite.kind {
            PermissionOverwriteType::Member => {
                added_member_ids.push(permission_overwrite.id.cast());
            }
            PermissionOverwriteType::Role => {
                if permission_overwrite
                    .id
                    .eq(&interaction.voice_channel.guild_id.cast())
                {
                    is_locked = permission_overwrite.deny.contains(Permissions::CONNECT);
                } else {
                    added_role_ids.push(permission_overwrite.id.cast());
                }
            }
            _ => continue,
        };
    }

    let members_text = if added_member_ids.is_empty() {
        "No members have been added to this voice channel.".to_owned()
    } else {
        let mut text = added_member_ids
            .iter()
            .map(|id| format!("- <@{id}>"))
            .collect::<Vec<String>>()
            .join("\n");

        if added_member_ids.len() > 5 {
            let remaining_members: Vec<Id<UserMarker>> = added_member_ids.split_off(5);

            text.push_str(&format!("\n +{} more", remaining_members.len()));
        }

        text
    };
    let roles_text = if added_role_ids.is_empty() {
        "No roles have been added to this voice channel.".to_owned()
    } else {
        let mut text = added_role_ids
            .iter()
            .map(|id| format!("- <@&{id}>"))
            .collect::<Vec<String>>()
            .join("\n");

        if added_role_ids.len() > 5 {
            let remaining_roles: Vec<Id<RoleMarker>> = added_role_ids.split_off(5);

            text.push_str(&format!("\n +{} more", remaining_roles.len()));
        }

        text
    };
    let locked_text = if is_locked {
        "This voice channel is currently locked.".to_owned()
    } else {
        "This voice channel is currently unlocked.".to_owned()
    };
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .field(EmbedFieldBuilder::new("Added member(s)", members_text).build())
        .field(EmbedFieldBuilder::new("Added role(s)", roles_text).build())
        .field(EmbedFieldBuilder::new("Locked state", locked_text).build())
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
