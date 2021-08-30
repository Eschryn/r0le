use serenity::{async_trait, prelude::*, model::prelude::*};

use crate::data::{RedisReactionRoleStore, ReactionRoleStore};

pub struct Handler(pub u64);

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::watching("~help")).await;
        println!("{} is connected!", ready.user.name);
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        if add_reaction.user_id == Some(UserId(self.0)) {
            return;
        }

        let rol = Self::get_store(&ctx).await
            .get(add_reaction.guild_id.unwrap(), add_reaction.channel_id, add_reaction.message_id, add_reaction.emoji);

        ctx.http.add_member_role(add_reaction.guild_id.unwrap().0, add_reaction.user_id.unwrap().0, rol).await.unwrap();
    }

    async fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        let store = Self::get_store(&ctx).await;
        
        if removed_reaction.user_id == Some(UserId(self.0)) {
            store.delete(removed_reaction.guild_id.unwrap(), Some(removed_reaction.channel_id), Some(removed_reaction.message_id), Some(removed_reaction.emoji));
        } else {
            let rol = store.get(removed_reaction.guild_id.unwrap(), removed_reaction.channel_id, removed_reaction.message_id, removed_reaction.emoji);
        
            ctx.http.remove_member_role(removed_reaction.guild_id.unwrap().0, removed_reaction.user_id.unwrap().0, rol).await.unwrap();
        }
    }

    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
        Self::get_store(&ctx).await
            .delete(guild_id.unwrap(), Some(channel_id), Some(deleted_message_id), None);
    }

    async fn channel_delete(&self, ctx: Context, channel: &GuildChannel) {
        Self::get_store(&ctx).await
            .delete(channel.guild_id, Some(channel.id), None, None);
    }

    async fn guild_delete(&self, ctx: Context, incomplete: GuildUnavailable, _full: Option<Guild>) {
        Self::get_store(&ctx).await
            .delete(incomplete.id, None, None, None);
    }
}

impl Handler {
    async fn get_store(ctx: &Context) -> RedisReactionRoleStore {
        let data = ctx.data.read().await;

        data.get::<RedisReactionRoleStore>().unwrap().clone()
    }
}