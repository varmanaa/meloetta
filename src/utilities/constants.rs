use std::{collections::HashSet, sync::LazyLock};

use std::env;

use eyre::WrapErr;
use twilight_gateway::{EventTypeFlags, Intents};
use twilight_model::{
    application::command::{Command, CommandType},
    channel::{
        message::{
            component::{ActionRow, SelectMenu, SelectMenuOption, SelectMenuType},
            Component, Embed,
        },
        ChannelType,
    },
};
use twilight_util::builder::{
    command::{BooleanBuilder, ChannelBuilder, CommandBuilder, StringBuilder, SubCommandBuilder},
    embed::EmbedBuilder,
};

pub static COMMANDS: LazyLock<Vec<Command>> = LazyLock::new(|| {
    vec![
        CommandBuilder::new(
            "create",
            "Create (or recreate) missing things",
            CommandType::ChatInput,
        )
        .option(
            SubCommandBuilder::new("join-channel", "Recreate a join channel")
                .option(
                    ChannelBuilder::new(
                        "category",
                        "The voice category to create the join channel for",
                    )
                    .channel_types(vec![ChannelType::GuildCategory])
                    .required(true)
                    .build(),
                )
                .build(),
        )
        .option(
            SubCommandBuilder::new("panel-message", "Recreate a panel message")
                .option(
                    ChannelBuilder::new(
                        "voice-channel",
                        "The voice channel to create the panel message for",
                    )
                    .channel_types(vec![ChannelType::GuildVoice])
                    .required(true)
                    .build(),
                )
                .build(),
        )
        .option(
            SubCommandBuilder::new("voice-category", "The voice category to create")
                .option(
                    StringBuilder::new("name", "The voice category name")
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .build(),
        CommandBuilder::new(
            "info",
            "View categories and settings",
            CommandType::ChatInput,
        )
        .build(),
        CommandBuilder::new(
            "remove-category",
            "Remove a voice category",
            CommandType::ChatInput,
        )
        .option(
            ChannelBuilder::new("category", "The voice category to remove")
                .channel_types(vec![ChannelType::GuildCategory])
                .required(true)
                .build(),
        )
        .build(),
        CommandBuilder::new(
            "permanence",
            "Set the permanence (if permanent or temporary) state of voice channels",
            CommandType::ChatInput,
        )
        .option(
            BooleanBuilder::new(
                "state",
                "Should voice channels remain if all users have left?",
            )
            .required(true),
        )
        .build(),
    ]
});

pub static DATABASE_URL: LazyLock<String> = LazyLock::new(|| {
    env::var("DATABASE_URL")
        .wrap_err("Environment variable \"DATABASE_URL\" is not set.")
        .unwrap()
});

pub static DISCORD_TOKEN: LazyLock<String> = LazyLock::new(|| {
    env::var("DISCORD_TOKEN")
        .wrap_err("Environment variable \"DISCORD_TOKEN\" is not set.")
        .unwrap()
});

pub static INTENTS: LazyLock<Intents> = LazyLock::new(|| {
    Intents::GUILDS
        | Intents::GUILD_MEMBERS
        | Intents::GUILD_MODERATION
        | Intents::GUILD_VOICE_STATES
});

pub static NON_VOICE_CHANNEL_OWNER_SELECT_OPTIONS: LazyLock<HashSet<String>> =
    LazyLock::new(|| HashSet::from_iter(vec!["claim-select-option".to_owned()]));

pub static PANEL_MESSAGE_COMPONENTS: LazyLock<Vec<Component>> = LazyLock::new(|| {
    let options = [
        ("Add member permissions", "add-member-select-option"),
        ("Add role permissions", "add-role-select-option"),
        ("Claim voice channel", "claim-select-option"),
        ("Kick member", "kick-member-select-option"),
        ("Lock channel", "lock-channel-select-option"),
        ("Modify bitrate", "modify-bitrate-select-option"),
        ("Modify name", "modify-name-select-option"),
        ("Modify slowmode", "modify-slowmode-select-option"),
        ("Modify user limit", "modify-user-limit-select-option"),
        ("Modify video quality", "modify-video-quality-select-option"),
        ("Remove channel", "remove-channel-select-option"),
        ("Remove member permissions", "remove-member-select-option"),
        ("Remove role permissions", "remove-role-select-option"),
        ("Transfer voice channel", "transfer-select-option"),
        ("Unlock channel", "unlock-channel-select-option"),
        ("View information", "view-information-select-option"),
    ]
    .into_iter()
    .map(|(label, value)| SelectMenuOption {
        default: false,
        description: None,
        emoji: None,
        label: label.to_owned(),
        value: value.to_owned(),
    })
    .collect::<Vec<SelectMenuOption>>();
    let select_menu = Component::ActionRow(ActionRow {
        components: vec![Component::SelectMenu(SelectMenu {
            channel_types: None,
            custom_id: "edit-channel-select".to_owned(),
            default_values: None,
            disabled: false,
            kind: SelectMenuType::Text,
            max_values: Some(1),
            min_values: Some(1),
            options: Some(options),
            placeholder: Some("Edit channel".to_owned()),
        })],
    });

    vec![select_menu]
});

pub static PANEL_MESSAGE_EMBED: LazyLock<Embed> = LazyLock::new(|| {
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description("Make the channel your own!")
        .build();

    embed
});

pub static WANTED_EVENT_TYPES: LazyLock<EventTypeFlags> = LazyLock::new(|| {
    EventTypeFlags::BAN_ADD
        | EventTypeFlags::CHANNEL_DELETE
        | EventTypeFlags::CHANNEL_UPDATE
        | EventTypeFlags::GUILD_CREATE
        | EventTypeFlags::GUILD_DELETE
        | EventTypeFlags::INTERACTION_CREATE
        | EventTypeFlags::MEMBER_REMOVE
        | EventTypeFlags::MESSAGE_DELETE
        | EventTypeFlags::READY
        | EventTypeFlags::ROLE_CREATE
        | EventTypeFlags::ROLE_DELETE
        | EventTypeFlags::ROLE_UPDATE
        | EventTypeFlags::UNAVAILABLE_GUILD
        | EventTypeFlags::VOICE_STATE_UPDATE
});
