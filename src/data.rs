use redis::Commands;
use serenity::{model::prelude::*, prelude::*};


pub trait ReactionRoleStore {
    fn create(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: ReactionType, role: RoleId);
    fn delete(&self, guild_id: GuildId, channel: Option<ChannelId>, message: Option<MessageId>, reaction: Option<ReactionType>);
    fn get(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: ReactionType) -> u64;
}

#[derive(Clone)]
pub struct RedisReactionRoleStore {
    pool: r2d2::Pool<redis::Client>
}

impl RedisReactionRoleStore {
    pub fn connect(url: String) -> Self {
        let redis_client = redis::Client::open(url).unwrap();
        let pool = r2d2::Pool::builder()
                .max_size(15)
                .build(redis_client)
                .unwrap();
        
        Self {
            pool
        }
    }
}

impl ReactionRoleStore for RedisReactionRoleStore {
    fn create(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: ReactionType, role: RoleId) {
        let mut connection = self.pool.get().unwrap();
        let key = format!("{}:{}:{}", guild_id.0, channel_id.0, message_id.0);

        let _: () = connection.hset(key, reaction.as_data(),  role.0).unwrap();
    }

    fn delete(&self, guild_id: GuildId, channel_id: Option<ChannelId>, message_id: Option<MessageId>, reaction: Option<ReactionType>) {
        let key = format!("{}:{}:{}", guild_id.0, channel_id.map(|a| a.0.to_string()).unwrap_or("*".to_string()), message_id.map(|a| a.0.to_string()).unwrap_or("*".to_string()));
        println!("Deleting {}", key);

        let mut connection = self.pool.get().unwrap();

        if let Some(reaction_type) = reaction {
            let _: () = connection.del(key + ":" + reaction_type.as_data().as_str()).unwrap();
        } else {
            let keys: Vec<String> = connection.scan_match(key).unwrap().collect();
            if !keys.is_empty() {
                let _: () = connection.del(keys).unwrap();
            }
        }
    }

    fn get(&self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId, reaction: ReactionType) -> u64 {
        let mut connection = self.pool.get().unwrap();
        let key = format!("{}:{}:{}", guild_id.0, channel_id.0, message_id.0);
        
        connection.hget(key, reaction.as_data()).unwrap()
    }
}

impl TypeMapKey for RedisReactionRoleStore {
    type Value = RedisReactionRoleStore;
}