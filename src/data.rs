use std::fmt::Debug;

use log::warn;
use redis::{Commands, RedisError};
use serenity::{model::prelude::*, prelude::*};
use thiserror::Error;

pub trait ReactionRoleStore {
    type Error: Debug;

    fn create(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: &ReactionType, role: RoleId) -> Result<(), Self::Error>;
    fn delete(&self, guild_id: GuildId, channel: Option<ChannelId>, message: Option<MessageId>, reaction: Option<ReactionType>) -> Result<(), Self::Error>;
    fn get(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: ReactionType) -> Result<u64, Self::Error>;
}

#[derive(Clone)]
pub struct RedisReactionRoleStore {
    pool: r2d2::Pool<redis::Client>
}

impl RedisReactionRoleStore {
    pub async fn connect(url: String) -> Result<Self, RedisReactionRoleStoreError> {
        let redis_client = redis::Client::open(url)?;
        let pool = r2d2::Pool::builder()
                .max_size(15)
                .build(redis_client)?;
        
        Ok(Self { pool })
    }
}

#[derive(Error, Debug)]
pub enum RedisReactionRoleStoreError {
    #[error(transparent)]
    RedisError(#[from] RedisError),
    #[error(transparent)]
    R2d2Error(#[from] r2d2::Error)
}

impl ReactionRoleStore for RedisReactionRoleStore {
    type Error = RedisReactionRoleStoreError;

    fn create(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: &ReactionType, role: RoleId) -> Result<(), Self::Error> {
        let mut connection = self.pool.get()?;
        let key = format!("{}:{}:{}", guild_id.0, channel_id.0, message_id.0);

        Ok(connection.hset(key, reaction.as_data(),  role.0)?)
    }

    fn delete(&self, guild_id: GuildId, channel_id: Option<ChannelId>, message_id: Option<MessageId>, reaction: Option<ReactionType>) -> Result<(), Self::Error> {
        let key = format!("{}:{}:{}", guild_id.0, channel_id.map(|a| a.0.to_string()).unwrap_or("*".to_string()), message_id.map(|a| a.0.to_string()).unwrap_or("*".to_string()));
        warn!("Deleting {}", key);

        let mut connection = self.pool.get()?;

        if let Some(reaction_type) = reaction {
            Ok(connection.del(key + ":" + reaction_type.as_data().as_str())?)
        } else {
            let keys: Vec<String> = connection.scan_match(key)?.collect();
            if !keys.is_empty() {
                Ok(connection.del(keys)?)
            } else {
                Ok(())
            }
        }
    }

    fn get(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: ReactionType) -> Result<u64, Self::Error> {
        let mut connection = self.pool.get()?;
        let key = format!("{}:{}:{}", guild_id.0, channel_id.0, message_id.0);
        
        Ok(connection.hget(key, reaction.as_data())?)
    }
}

impl TypeMapKey for RedisReactionRoleStore {
    type Value = RedisReactionRoleStore;
}