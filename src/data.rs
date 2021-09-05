use std::fmt::Debug;

use log::warn;
use redis::{AsyncCommands, RedisError, aio::ConnectionManager};
use serenity::{async_trait, model::prelude::*, prelude::*};
use thiserror::Error;
use futures_util::StreamExt;

#[async_trait]
pub trait ReactionRoleStore {
    type Error: Debug;

    async fn create(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: &ReactionType, role: RoleId) -> Result<(), Self::Error>;
    async fn delete(&self, guild_id: GuildId, channel: Option<ChannelId>, message: Option<MessageId>, reaction: Option<ReactionType>) -> Result<(), Self::Error>;
    async fn get(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: ReactionType) -> Result<u64, Self::Error>;
}

#[derive(Clone)]
pub struct RedisReactionRoleStore {
    connection_manager: ConnectionManager
}

impl RedisReactionRoleStore {
    pub async fn connect(url: String) -> Result<Self, RedisReactionRoleStoreError> {
        let connection_manager = redis::Client::open(url)?
            .get_tokio_connection_manager()
            .await?;

        Ok(Self { connection_manager })
    }
}

#[derive(Error, Debug)]
pub enum RedisReactionRoleStoreError {
    #[error(transparent)]
    RedisError(#[from] RedisError)
}

#[async_trait]
impl ReactionRoleStore for RedisReactionRoleStore {
    type Error = RedisReactionRoleStoreError;

    async fn create(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: &ReactionType, role: RoleId) -> Result<(), Self::Error> {
        let key = format!("{}:{}:{}", guild_id.0, channel_id.0, message_id.0);

        Ok(self.connection_manager.clone().hset(key, reaction.as_data(),  role.0).await?)
    }

    async fn delete(&self, guild_id: GuildId, channel_id: Option<ChannelId>, message_id: Option<MessageId>, reaction: Option<ReactionType>) -> Result<(), Self::Error> {
        let key = format!("{}:{}:{}", guild_id.0, channel_id.map(|a| a.0.to_string()).unwrap_or("*".to_string()), message_id.map(|a| a.0.to_string()).unwrap_or("*".to_string()));
        warn!("Deleting {}", key);

        let mut connection = self.connection_manager.clone();
        if let Some(reaction_type) = reaction {
            Ok(connection.del(key + ":" + reaction_type.as_data().as_str()).await?)
        } else {
            let keys: Vec<String> = connection.scan_match(key).await?.collect().await;
            if !keys.is_empty() {
                Ok(connection.del(keys).await?)
            } else {
                Ok(())
            }
        }
    }

    async fn get(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: ReactionType) -> Result<u64, Self::Error> {
        let key = format!("{}:{}:{}", guild_id.0, channel_id.0, message_id.0);
        
        Ok(self.connection_manager.clone().hget(key, reaction.as_data()).await?)
    }
}

impl TypeMapKey for RedisReactionRoleStore {
    type Value = RedisReactionRoleStore;
}