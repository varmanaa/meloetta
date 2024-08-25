use std::str::FromStr;

use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use eyre::{Context, Result};
use futures::future::try_join_all;
use tokio_postgres::{types::ToSql, Config, NoTls, Row};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker},
    Id,
};

use crate::utilities::constants::DATABASE_URL;

pub struct Database {
    pub pool: Pool,
}

pub struct DatabaseCategoryChannel {
    #[allow(dead_code)]
    pub guild_id: Id<GuildMarker>,
    pub id: Id<ChannelMarker>,
    pub join_channel_id: Option<Id<ChannelMarker>>,
}

pub struct DatabaseGuild {
    #[allow(dead_code)]
    pub id: Id<GuildMarker>,
    pub permanence: bool,
    pub privacy: String,
}

pub struct DatabaseVoiceChannel {
    pub id: Id<ChannelMarker>,
    #[allow(dead_code)]
    pub guild_id: Id<GuildMarker>,
    pub parent_id: Id<ChannelMarker>,
    pub owner_id: Option<Id<UserMarker>>,
    pub panel_message_id: Option<Id<MessageMarker>>,
}

impl Database {
    pub async fn create_tables(&self) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            -- guild table
            CREATE TABLE IF NOT EXISTS public.guild (
                id INT8 PRIMARY KEY,
                permanence BOOLEAN NOT NULL DEFAULT FALSE,
                privacy TEXT NOT NULL DEFAULT 'unlocked'
            );

            -- category_channel table
            CREATE TABLE IF NOT EXISTS public.category_channel (
                id INT8 PRIMARY KEY,
                guild_id INT8 NOT NULL REFERENCES public.guild(id) ON DELETE CASCADE,
                join_channel_id INT8
            );

            -- voice_channel table
            CREATE TABLE IF NOT EXISTS public.voice_channel (
                id INT8 PRIMARY KEY NOT NULL,
                guild_id INT8 NOT NULL REFERENCES public.guild(id) ON DELETE CASCADE,
                parent_id INT8 NOT NULL REFERENCES public.category_channel(id) ON DELETE CASCADE,
                owner_id INT8,
                panel_message_id INT8
            );
        ";

        client.batch_execute(statement).await?;

        Ok(())
    }

    pub fn new() -> Self {
        Self {
            pool: Pool::builder(Manager::from_config(
                Config::from_str(DATABASE_URL.as_str()).unwrap(),
                NoTls,
                ManagerConfig {
                    recycling_method: RecyclingMethod::Fast,
                },
            ))
            .max_size(16)
            .build()
            .wrap_err("Unable to create connection pool.")
            .unwrap(),
        }
    }

    pub async fn guild(&self, guild_id: Id<GuildMarker>) -> Result<Option<DatabaseGuild>> {
        let client = self.pool.get().await?;
        let statement = "
            SELECT
                *
            FROM
                guild
            WHERE
                id = $1;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[&(guild_id.get() as i64)];
        let row = client
            .query_one(statement, params)
            .await
            .wrap_err("Unable to run \"guild\" endpoint");
        let guild = row.ok().map(|row| DatabaseGuild::from(row));

        Ok(guild)
    }

    pub async fn insert_guild(&self, guild_id: Id<GuildMarker>) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            INSERT INTO
                guild (id)
            VALUES
                ($1)
            ON CONFLICT
            DO NOTHING;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[&(guild_id.get() as i64)];

        client
            .execute(statement, params)
            .await
            .wrap_err("Unable to run \"insert_guild\" endpoint")?;

        Ok(())
    }

    pub async fn remove_channels(
        &self,
        guild_id: Id<GuildMarker>,
        channel_ids: Vec<Id<ChannelMarker>>,
    ) -> Result<Vec<Id<ChannelMarker>>> {
        let client = self.pool.get().await?;
        let statements = vec![
            "
                DELETE FROM
                    category_channel
                WHERE
                    guild_id = $1
                    AND NOT(id = ANY($2::INT8[]))
                RETURNING
                    id;
            ",
            "
                DELETE FROM
                    voice_channel
                WHERE
                    guild_id = $1
                    AND NOT(id = ANY($2::INT8[]))
                RETURNING
                    id;
            ",
        ];
        let params: &[&(dyn ToSql + Sync)] = &[
            &(guild_id.get() as i64),
            &channel_ids
                .into_iter()
                .map(|id| id.get() as i64)
                .collect::<Vec<i64>>(),
        ];
        let prepared_statements = try_join_all(
            statements
                .into_iter()
                .map(|statement| client.prepare(&statement)),
        )
        .await?;
        let results = try_join_all(
            prepared_statements
                .iter()
                .map(|prepared_statement| client.query(prepared_statement, params)),
        )
        .await
        .wrap_err("Unable to run \"remove_channels\" endpoint")?;
        let removed_channel_ids = results
            .into_iter()
            .flatten()
            .map(|row| Id::new(row.get::<_, i64>("id") as u64))
            .collect::<Vec<Id<ChannelMarker>>>();

        Ok(removed_channel_ids)
    }

    pub async fn update_channels(
        &self,
        guild_id: Id<GuildMarker>,
        voice_channel_and_parent_ids: Vec<(Id<ChannelMarker>, Id<ChannelMarker>)>,
    ) -> Result<()> {
        let client = self.pool.get().await?;
        let values_param = voice_channel_and_parent_ids
            .into_iter()
            .map(|(id, parent_id)| format!("({}, {})", id.get() as i64, parent_id.get() as i64))
            .collect::<Vec<String>>()
            .join(",");
        let statement = format!(
            "
            UPDATE
                voice_channel
            SET
                parent_id = channel.parent_id
            FROM 
                (VALUES {values_param}) AS channel(id, parent_id)
            WHERE
                voice_channel.guild_id = $1
                AND voice_channel.id = channel.id;
        "
        );
        let params: &[&(dyn ToSql + Sync)] = &[&(guild_id.get() as i64)];

        client
            .execute(&statement, params)
            .await
            .wrap_err("Unable to run \"update_channels\" endpoint")?;

        Ok(())
    }

    pub async fn update_panel_message(
        &self,
        voice_channel_id: Id<ChannelMarker>,
        panel_message_id: Option<Id<MessageMarker>>,
    ) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = format!(
            "
            UPDATE
                voice_channel
            SET
                panel_message_id = $2
            WHERE
                id = $1;
        "
        );
        let params: &[&(dyn ToSql + Sync)] = &[
            &(voice_channel_id.get() as i64),
            &(panel_message_id.map(|id| id.get() as i64)),
        ];

        client
            .execute(&statement, params)
            .await
            .wrap_err("Unable to run \"update_panel_message\" endpoint")?;

        Ok(())
    }

    pub async fn guild_category_channels(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Vec<DatabaseCategoryChannel>> {
        let client = self.pool.get().await?;
        let statement = "
            SELECT
                *
            FROM
                category_channel
            WHERE
                guild_id = $1;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[&(guild_id.get() as i64)];
        let rows = client
            .query(statement, params)
            .await
            .wrap_err("Unable to run \"guild_category_channels\" endpoint");
        let guild_category_channels = rows
            .unwrap_or_default()
            .into_iter()
            .map(|row| DatabaseCategoryChannel::from(row))
            .collect::<Vec<DatabaseCategoryChannel>>();

        Ok(guild_category_channels)
    }

    pub async fn guild_voice_channels(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Vec<DatabaseVoiceChannel>> {
        let client = self.pool.get().await?;
        let statement = "
            SELECT
                *
            FROM
                voice_channel 
            WHERE
                guild_id = $1;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[&(guild_id.get() as i64)];
        let rows = client
            .query(statement, params)
            .await
            .wrap_err("Unable to run \"guild_voice_channels\" endpoint");
        let guild_voice_channels = rows
            .unwrap_or_default()
            .into_iter()
            .map(|row| DatabaseVoiceChannel::from(row))
            .collect::<Vec<DatabaseVoiceChannel>>();

        Ok(guild_voice_channels)
    }

    pub async fn insert_voice_channel(
        &self,
        id: Id<ChannelMarker>,
        guild_id: Id<GuildMarker>,
        parent_id: Id<ChannelMarker>,
        owner_id: Id<UserMarker>,
    ) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            INSERT INTO
                voice_channel
            VALUES
                ($1, $2, $3, $4)
            ON CONFLICT
            DO NOTHING;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[
            &(id.get() as i64),
            &(guild_id.get() as i64),
            &(parent_id.get() as i64),
            &(owner_id.get() as i64),
        ];

        client
            .execute(statement, params)
            .await
            .wrap_err("Unable to run \"insert_category_channel\" endpoint")?;

        Ok(())
    }

    pub async fn remove_voice_channel(&self, channel_id: Id<ChannelMarker>) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            DELETE FROM
                voice_channel
            WHERE
                id = $1;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[&(channel_id.get() as i64)];

        client
            .execute(statement, params)
            .await
            .wrap_err("Unable to run \"remove_voice_channel\" endpoint")?;

        Ok(())
    }

    pub async fn remove_guild(&self, guild_id: Id<GuildMarker>) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            DELETE FROM
                guild
            WHERE
                id = $1;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[&(guild_id.get() as i64)];

        client
            .execute(statement, params)
            .await
            .wrap_err("Unable to run \"remove_guild\" endpoint")?;

        Ok(())
    }

    pub async fn remove_category_channel(&self, channel_id: Id<ChannelMarker>) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            DELETE FROM
                category_channel
            WHERE
                id = $1;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[&(channel_id.get() as i64)];

        client
            .execute(statement, params)
            .await
            .wrap_err("Unable to run \"remove_category_channel\" endpoint")?;

        Ok(())
    }

    pub async fn update_join_channel(
        &self,
        channel_id: Id<ChannelMarker>,
        join_channel_id: Option<Id<ChannelMarker>>,
    ) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            UPDATE
                category_channel
            SET
                join_channel_id = $2
            WHERE
                id = $1;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[
            &(channel_id.get() as i64),
            &(join_channel_id.map(|id| id.get() as i64)),
        ];

        client
            .execute(statement, params)
            .await
            .wrap_err("Unable to run \"update_join_channel\" endpoint.")?;

        Ok(())
    }

    pub async fn update_permanence(
        &self,
        guild_id: Id<GuildMarker>,
        permanence: bool,
    ) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            UPDATE
                guild
            SET
                permanence = $2
            WHERE
                id = $1;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[&(guild_id.get() as i64), &permanence];

        client
            .execute(statement, params)
            .await
            .wrap_err("Unable to run \"update_permanence\" endpoint.")?;

        Ok(())
    }

    pub async fn update_privacy(&self, guild_id: Id<GuildMarker>, privacy: String) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            UPDATE
                guild
            SET
                privacy = $2
            WHERE
                id = $1;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[&(guild_id.get() as i64), &privacy];

        client
            .execute(statement, params)
            .await
            .wrap_err("Unable to run \"update_privacy\" endpoint.")?;

        Ok(())
    }

    pub async fn update_voice_channel_owner(
        &self,
        voice_channel_id: Id<ChannelMarker>,
        owner_id: Option<Id<UserMarker>>,
    ) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            UPDATE
                voice_channel
            SET
                owner_id = $2
            WHERE
                id = $1;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[
            &(voice_channel_id.get() as i64),
            &(owner_id.map(|id| id.get() as i64)),
        ];

        client
            .execute(statement, params)
            .await
            .wrap_err("Unable to run \"update_voice_channel_owner\" endpoint.")?;

        Ok(())
    }

    pub async fn insert_category_channel(
        &self,
        id: Id<ChannelMarker>,
        guild_id: Id<GuildMarker>,
        join_channel_id: Option<Id<ChannelMarker>>,
    ) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            INSERT INTO
                category_channel
            VALUES
                ($1, $2, $3)
            ON CONFLICT
            DO NOTHING;
        ";
        let params: &[&(dyn ToSql + Sync)] = &[
            &(id.get() as i64),
            &(guild_id.get() as i64),
            &(join_channel_id.map(|id| id.get() as i64)),
        ];

        client
            .execute(statement, params)
            .await
            .wrap_err("Unable to run \"insert_category_channel\" endpoint")?;

        Ok(())
    }
}

impl From<Row> for DatabaseCategoryChannel {
    fn from(row: Row) -> Self {
        Self {
            guild_id: Id::new(row.get::<_, i64>("guild_id") as u64),
            id: Id::new(row.get::<_, i64>("id") as u64),
            join_channel_id: row
                .try_get::<_, i64>("join_channel_id")
                .map_or(None, |id| Some(Id::new(id as u64))),
        }
    }
}

impl From<Row> for DatabaseGuild {
    fn from(row: Row) -> Self {
        Self {
            id: Id::new(row.get::<_, i64>("id") as u64),
            permanence: row.get::<_, bool>("permanence"),
            privacy: row.get::<_, String>("privacy"),
        }
    }
}

impl From<Row> for DatabaseVoiceChannel {
    fn from(row: Row) -> Self {
        Self {
            id: Id::new(row.get::<_, i64>("id") as u64),
            guild_id: Id::new(row.get::<_, i64>("guild_id") as u64),
            parent_id: Id::new(row.get::<_, i64>("parent_id") as u64),
            owner_id: row
                .try_get::<_, i64>("owner_id")
                .map_or(None, |id| Some(Id::new(id as u64))),
            panel_message_id: row
                .try_get::<_, i64>("panel_message_id")
                .map_or(None, |id| Some(Id::new(id as u64))),
        }
    }
}
