use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
    guild::Permissions,
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

    let mut voice_channel_permission_overwrites = interaction
        .voice_channel
        .permission_overwrites
        .read()
        .clone();
    let is_locked = voice_channel_permission_overwrites
        .iter()
        .find(|permission_overwrite| {
            permission_overwrite
                .id
                .eq(&interaction.voice_channel.guild_id.cast())
                && permission_overwrite.kind.eq(&PermissionOverwriteType::Role)
        })
        .is_some_and(|everyone_permission_overwrite| {
            everyone_permission_overwrite
                .deny
                .contains(Permissions::CONNECT)
        });
    let description = if is_locked {
        "This voice channel is already locked!"
    } else {
        let bot_id: Id<GenericMarker> = context.application_id.cast();
        let voice_channel_owner_id = interaction.voice_channel.owner_id.read().clone();
        let everyone_role_id: Id<GenericMarker> = interaction.voice_channel.guild_id.cast();
        let (mut bot_allow, mut bot_deny): (Permissions, Permissions) =
            (Permissions::empty(), Permissions::empty());
        let (mut voice_channel_owner_allow, mut voice_channel_owner_deny): (
            Permissions,
            Permissions,
        ) = (Permissions::empty(), Permissions::empty());
        let (mut everyone_allow, mut everyone_deny): (Permissions, Permissions) =
            (Permissions::empty(), Permissions::empty());

        voice_channel_permission_overwrites.retain(
            |permission_overwrite| match permission_overwrite.kind {
                PermissionOverwriteType::Member if bot_id.eq(&permission_overwrite.id) => {
                    bot_allow = permission_overwrite.allow;
                    bot_deny = permission_overwrite.deny;

                    false
                }
                PermissionOverwriteType::Member
                    if voice_channel_owner_id.is_some_and(|owner_id| {
                        owner_id.get().eq(&permission_overwrite.id.get())
                    }) =>
                {
                    voice_channel_owner_allow = permission_overwrite.allow;
                    voice_channel_owner_deny = permission_overwrite.deny;

                    false
                }
                PermissionOverwriteType::Role if everyone_role_id.eq(&permission_overwrite.id) => {
                    everyone_allow = permission_overwrite.allow;
                    everyone_deny = permission_overwrite.deny;

                    false
                }
                _ => {
                    !permission_overwrite.allow.is_empty() || !permission_overwrite.deny.is_empty()
                }
            },
        );

        everyone_allow.remove(Permissions::CONNECT);
        everyone_deny.insert(Permissions::CONNECT);
        bot_allow.insert(Permissions::CONNECT);
        voice_channel_owner_allow.insert(Permissions::CONNECT);

        voice_channel_permission_overwrites.extend(vec![
            PermissionOverwrite {
                allow: everyone_allow,
                deny: everyone_deny,
                id: everyone_role_id,
                kind: PermissionOverwriteType::Role,
            },
            PermissionOverwrite {
                allow: bot_allow,
                deny: bot_deny,
                id: bot_id,
                kind: PermissionOverwriteType::Member,
            },
        ]);

        if let Some(owner_id) = voice_channel_owner_id {
            voice_channel_permission_overwrites.extend(vec![PermissionOverwrite {
                allow: voice_channel_owner_allow,
                deny: voice_channel_owner_deny,
                id: owner_id.cast(),
                kind: PermissionOverwriteType::Member,
            }]);
        }

        context
            .client
            .update_channel(interaction.voice_channel.id)
            .permission_overwrites(&voice_channel_permission_overwrites)
            .await?;

        "I've locked the voice channel!"
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
