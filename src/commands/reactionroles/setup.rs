use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}, time::Duration};

use futures::StreamExt;
use r0le::WorkerMenu;
use serenity::{builder::{CreateButton, CreateMessage}, framework::standard::{Args, CommandResult, macros::command}, model::{interactions::message_component::{ButtonStyle, MessageComponentInteraction}, prelude::*}, prelude::*};
use log::error;
use serenity::async_trait;

use crate::data::{RedisReactionRoleStore, ReactionRoleStore};

macro_rules! create_role_menu_embed {
    ($roles:expr, $pos:expr) => {
        |create_embed| {
            create_embed.title("Reaction role setup")
                .description("React to the messages with the emojis that should trigger the reaction roles.");
                
            if let Some(role) = $roles.get($pos) {
                create_embed.field("current role", role.mention(), true);
            }
    
            create_embed.footer(|f| f.text(format!("{}/{}", $pos, $roles.len())))
        }
    };
}

#[command]
async fn setup(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let roles: Vec<RoleId> = args.iter::<RoleId>().filter_map(|r| r.ok()).collect();
    let pool = {
        let data = ctx.data.read().await;

        data.get::<RedisReactionRoleStore>().unwrap().clone()
    };

    if roles.len() == 0 {
        return Ok(());
    }
    
    let guild = msg.guild(ctx).await;
    if guild.is_none() {
        println!("no guild!");
        return Ok(());
    }

    let guild = guild.unwrap();

    ReactionRoleAssignmentMenu {
        author: msg.author.id.0, 
        roles, 
        pool, 
        guild,
        role_assignment_state: AtomicUsize::new(1)
    }.open(&ctx, &msg).await?;

    Ok(())
}

#[async_trait]
impl WorkerMenu for ReactionRoleAssignmentMenu {

    async fn process(&self, ctx: &Context, menu: &Message) {
        let mut reactions = self.guild.await_reactions(ctx)
            .author_id(self.author)
            .guild_id(self.guild.id)
            .collect_limit(self.roles.len() as u32)
            .timeout(Duration::from_secs(60 * 5))
            .await
            .enumerate();

        let mut pos = self.role_assignment_state.load(Ordering::Relaxed) - 1;
        while let Some((_, reaction_action)) = reactions.next().await {
            let reaction = reaction_action.as_inner_ref();

            match self.pool.create(reaction.guild_id.unwrap(), reaction.channel_id, reaction.message_id, &reaction.emoji, self.roles[pos]) {
                Ok(_) => {
                    pos = self.role_assignment_state.fetch_add(1, Ordering::Relaxed);
                    menu.clone().edit(ctx, |m| m.embed(create_role_menu_embed!(&self.roles, pos))).await.unwrap(); 
            
                    ctx.http.create_reaction(reaction.channel_id.0, reaction.message_id.0, &reaction.emoji).await.unwrap();
        
                    if pos == self.roles.len() {
                        break;
                    }
                }
                Err(e) => {
                    error!("{}", e);

                    let _ = menu.clone()
                        .edit(ctx, |m| m.content("an error occurred while trying to add reaction handler - please contact the developer to resolve the issue"))
                        .await; 
                }
            } 
        }
    }
    
    async fn ui_event(&self, ctx: &Context, interaction: &Arc<MessageComponentInteraction>) -> bool {
        match interaction.data.custom_id.parse().unwrap() {
            Self::CANCEL_ID => {
                interaction.create_interaction_response(ctx, |cir| {
                    cir.kind(InteractionResponseType::DeferredUpdateMessage)
                }).await.unwrap();

                true
            },
            Self::SKIP_ID => {
                let i = self.role_assignment_state.fetch_add(1, Ordering::Relaxed);

                if i != self.roles.len() {
                    interaction.create_interaction_response(ctx, |cir| {
                        cir.kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|m| m.create_embed(create_role_menu_embed!(&self.roles, i)))
                    }).await.unwrap();
                    
                }
                
                i == self.roles.len()
            }
            _ => false
        }
    }

    fn create_ui<'a, 'b>(&self, m: &'a mut CreateMessage<'b>) -> &'a mut CreateMessage<'b>  {
        m.components(|c| {
            c.create_action_row(|a| {
                a.create_button(Self::skip_button_factory);
                a.create_button(Self::cancel_button_factory)
            })
        });

        m.embed(create_role_menu_embed!(&self.roles, 0))
    }
}

struct ReactionRoleAssignmentMenu {
    author: u64,
    roles: Vec<RoleId>,
    pool: RedisReactionRoleStore,
    guild: Guild,
    role_assignment_state: AtomicUsize
}

impl ReactionRoleAssignmentMenu {
    const CANCEL_ID: i32 = 12783;
    const SKIP_ID: i32 = 23847;

    fn cancel_button_factory(b: &mut CreateButton) -> &mut CreateButton {
        b.label("cancel");
        b.style(ButtonStyle::Danger);
        b.custom_id(Self::CANCEL_ID)
    }
    
    fn skip_button_factory(b: &mut CreateButton) -> &mut CreateButton {
        b.label("skip");
        b.style(ButtonStyle::Secondary);
        b.custom_id(Self::SKIP_ID)
    }
}
