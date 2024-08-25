use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use parking_lot::RwLock;
use twilight_model::{
    channel::permission_overwrite::PermissionOverwrite,
    id::{
        marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker},
        Id,
    },
};

pub struct Cache {
    category_channels: RwLock<HashMap<Id<ChannelMarker>, Arc<CachedCategoryChannel>>>,
    guilds: RwLock<HashMap<Id<GuildMarker>, Arc<CachedGuild>>>,
    unavailable_guilds: RwLock<HashSet<Id<GuildMarker>>>,
    voice_channels: RwLock<HashMap<Id<ChannelMarker>, Arc<CachedVoiceChannel>>>,
    voice_channel_owners:
        RwLock<HashMap<(Id<GuildMarker>, Id<UserMarker>), Arc<Id<ChannelMarker>>>>,
    voice_states: RwLock<HashMap<(Id<GuildMarker>, Id<UserMarker>), Arc<Id<ChannelMarker>>>>,
}

pub struct CachedCategoryChannel {
    pub guild_id: Id<GuildMarker>,
    pub id: Id<ChannelMarker>,
    pub join_channel_id: RwLock<Option<Id<ChannelMarker>>>,
    pub permission_overwrites: RwLock<Vec<PermissionOverwrite>>,
    pub voice_channel_ids: RwLock<HashSet<Id<ChannelMarker>>>,
}

pub struct CachedGuild {
    pub bot_role_id: Id<RoleMarker>,
    pub category_channel_ids: RwLock<HashSet<Id<ChannelMarker>>>,
    pub id: Id<GuildMarker>,
    pub permanence: RwLock<bool>,
    pub privacy: RwLock<String>,
}

#[derive(Debug)]
pub struct CachedVoiceChannel {
    pub connected_user_ids: RwLock<HashSet<Id<UserMarker>>>,
    pub guild_id: Id<GuildMarker>,
    pub id: Id<ChannelMarker>,
    pub owner_id: RwLock<Option<Id<UserMarker>>>,
    pub panel_message_id: RwLock<Option<Id<MessageMarker>>>,
    pub parent_id: Id<ChannelMarker>,
    pub permission_overwrites: RwLock<Vec<PermissionOverwrite>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            category_channels: RwLock::new(HashMap::new()),
            guilds: RwLock::new(HashMap::new()),
            unavailable_guilds: RwLock::new(HashSet::new()),
            voice_channel_owners: RwLock::new(HashMap::new()),
            voice_channels: RwLock::new(HashMap::new()),
            voice_states: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert_unavailable_guild(&self, id: Id<GuildMarker>) {
        self.unavailable_guilds.write().insert(id);
    }

    pub fn insert_guild(
        &self,
        id: Id<GuildMarker>,
        bot_role_id: Id<RoleMarker>,
        permanence: bool,
        privacy: String,
    ) {
        self.unavailable_guilds.write().remove(&id);
        self.guilds.write().insert(
            id,
            Arc::new(CachedGuild {
                bot_role_id,
                category_channel_ids: RwLock::new(HashSet::new()),
                id,
                permanence: RwLock::new(permanence),
                privacy: RwLock::new(privacy),
            }),
        );
    }

    pub fn insert_category_channel(
        &self,
        guild_id: Id<GuildMarker>,
        id: Id<ChannelMarker>,
        join_channel_id: Option<Id<ChannelMarker>>,
        permission_overwrites: Vec<PermissionOverwrite>,
        voice_channel_ids: impl IntoIterator<Item = Id<ChannelMarker>>,
    ) {
        if let Some(guild) = self.guild(guild_id) {
            guild.category_channel_ids.write().insert(id);
        }

        self.category_channels.write().insert(
            id,
            Arc::new(CachedCategoryChannel {
                guild_id,
                id,
                join_channel_id: RwLock::new(join_channel_id),
                permission_overwrites: RwLock::new(permission_overwrites),
                voice_channel_ids: RwLock::new(HashSet::from_iter(voice_channel_ids)),
            }),
        );
    }

    pub fn guild(&self, guild_id: Id<GuildMarker>) -> Option<Arc<CachedGuild>> {
        self.guilds.read().get(&guild_id).cloned()
    }

    pub fn insert_voice_channel(
        &self,
        connected_user_ids: impl IntoIterator<Item = Id<UserMarker>>,
        guild_id: Id<GuildMarker>,
        id: Id<ChannelMarker>,
        owner_id: Option<Id<UserMarker>>,
        panel_message_id: Option<Id<MessageMarker>>,
        parent_id: Id<ChannelMarker>,
        permission_overwrites: Vec<PermissionOverwrite>,
    ) {
        if let Some(category_channel) = self.category_channel(parent_id) {
            category_channel.voice_channel_ids.write().insert(id);
        };

        self.voice_channels.write().insert(
            id,
            Arc::new(CachedVoiceChannel {
                connected_user_ids: RwLock::new(HashSet::from_iter(connected_user_ids)),
                guild_id,
                id,
                owner_id: RwLock::new(owner_id),
                panel_message_id: RwLock::new(panel_message_id),
                parent_id,
                permission_overwrites: RwLock::new(permission_overwrites),
            }),
        );

        if let Some(owner_id) = owner_id {
            self.voice_channel_owners
                .write()
                .insert((guild_id, owner_id), Arc::new(id));
        }
    }

    pub fn category_channel(
        &self,
        channel_id: Id<ChannelMarker>,
    ) -> Option<Arc<CachedCategoryChannel>> {
        self.category_channels.read().get(&channel_id).cloned()
    }

    pub fn insert_voice_state(
        &self,
        guild_id: Id<GuildMarker>,
        channel_id: Id<ChannelMarker>,
        user_id: Id<UserMarker>,
    ) {
        if let Some(voice_channel) = self.voice_channel(channel_id) {
            voice_channel.connected_user_ids.write().insert(user_id);
        }

        self.voice_states
            .write()
            .insert((guild_id, user_id), Arc::new(channel_id));
    }

    pub fn voice_state(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> Option<Arc<Id<ChannelMarker>>> {
        self.voice_states.read().get(&(guild_id, user_id)).cloned()
    }

    pub fn voice_channel(&self, channel_id: Id<ChannelMarker>) -> Option<Arc<CachedVoiceChannel>> {
        self.voice_channels.read().get(&channel_id).cloned()
    }

    pub fn remove_voice_state(&self, guild_id: Id<GuildMarker>, user_id: Id<UserMarker>) {
        if let Some(voice_channel_id) = self.voice_states.write().remove(&(guild_id, user_id)) {
            if let Some(voice_channel) = self.voice_channel(*voice_channel_id) {
                voice_channel.connected_user_ids.write().remove(&user_id);
            }
        }
    }

    pub fn voice_channel_owner(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> Option<Arc<Id<ChannelMarker>>> {
        self.voice_channel_owners
            .read()
            .get(&(guild_id, user_id))
            .cloned()
    }

    pub fn remove_guild(&self, guild_id: Id<GuildMarker>) {
        if let Some(guild) = self.guilds.write().remove(&guild_id) {
            for category_channel_id in guild.category_channel_ids.read().iter() {
                self.remove_category_channel(*category_channel_id);
            }
        }
    }

    pub fn remove_voice_channel(&self, channel_id: Id<ChannelMarker>) {
        if let Some(voice_channel) = self.voice_channels.write().remove(&channel_id) {
            if let Some(owner_id) = voice_channel.owner_id.read().clone() {
                self.voice_channel_owners
                    .write()
                    .remove(&(voice_channel.guild_id, owner_id));
            }

            for connected_user_id in voice_channel.connected_user_ids.read().iter() {
                self.voice_states
                    .write()
                    .remove(&(voice_channel.guild_id, *connected_user_id));
            }

            if let Some(category_channel) = self.category_channel(voice_channel.parent_id) {
                category_channel
                    .voice_channel_ids
                    .write()
                    .remove(&channel_id);
            }
        };
    }

    pub fn remove_category_channel(&self, channel_id: Id<ChannelMarker>) {
        if let Some(category_channel) = self.category_channels.write().remove(&channel_id) {
            for voice_channel_id in category_channel.voice_channel_ids.read().iter() {
                self.remove_voice_channel(*voice_channel_id);
            }

            if let Some(guild) = self.guild(category_channel.guild_id) {
                guild.category_channel_ids.write().remove(&channel_id);
            }
        }
    }

    pub fn update_panel_message(
        &self,
        channel_id: Id<ChannelMarker>,
        panel_message_id: Option<Id<MessageMarker>>,
    ) {
        if let Some(voice_channel) = self.voice_channel(channel_id) {
            *voice_channel.panel_message_id.write() = panel_message_id;
        }
    }

    pub fn update_permanence(&self, guild_id: Id<GuildMarker>, permanence: bool) {
        if let Some(guild) = self.guild(guild_id) {
            *guild.permanence.write() = permanence;
        }
    }

    pub fn update_privacy(&self, guild_id: Id<GuildMarker>, privacy: String) {
        if let Some(guild) = self.guild(guild_id) {
            *guild.privacy.write() = privacy;
        }
    }

    pub fn update_join_channel(
        &self,
        channel_id: Id<ChannelMarker>,
        join_channel_id: Option<Id<ChannelMarker>>,
    ) {
        if let Some(category_channel) = self.category_channel(channel_id) {
            *category_channel.join_channel_id.write() = join_channel_id;
        }
    }

    pub fn update_voice_channel_owner(
        &self,
        channel_id: Id<ChannelMarker>,
        owner_id: Option<Id<UserMarker>>,
    ) {
        if let Some(voice_channel) = self.voice_channel(channel_id) {
            if let Some(current_owner_id) = voice_channel.owner_id.read().clone() {
                self.voice_channel_owners
                    .write()
                    .remove(&(voice_channel.guild_id, current_owner_id));
            }
            if let Some(new_owner_id) = owner_id {
                self.voice_channel_owners.write().insert(
                    (voice_channel.guild_id, new_owner_id),
                    Arc::new(voice_channel.id),
                );
            }

            *voice_channel.owner_id.write() = owner_id;
        }
    }
}
