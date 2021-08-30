use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}, time::Duration};

use futures::{StreamExt, future::{self, AbortHandle, Abortable}};
use serenity::{builder::CreateEmbed, framework::standard::{Args, CommandResult, macros::command}, model::{interactions::message_component::ButtonStyle, prelude::*}, prelude::*};

use crate::data::{RedisReactionRoleStore, ReactionRoleStore};

const CANCEL: i32 = 12783;
const SKIP: i32 = 23847;

// TODO: clean this stuff up
#[command]
async fn setup(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let roles: Vec<RoleId> = args.iter::<RoleId>().filter_map(|r| r.ok()).collect();
    let pool = {
        let data = ctx.data.read().await;

        data.get::<RedisReactionRoleStore>().unwrap().clone()
    };

    let guild = msg.guild(ctx).await;
    if guild.is_none() {
        println!("no guild!");
        return Ok(());
    }

    fn create_role_menu_embed<'a>(create_embed: &'a mut CreateEmbed, roles: &Vec<RoleId>, pos: usize) -> &'a mut CreateEmbed {
        create_embed.title("Reaction role setup")
            .description("React to the messages with the emojis that should trigger the reaction roles.");
            
        if let Some(r) = roles.get(pos) {
            create_embed.field("current role", r.mention(), true);
        }

        create_embed.footer(|f| f.text(format!("{}/{}", pos, roles.len())))
    }

    let rep = msg.channel_id.send_message(ctx, |m| {
        m.reference_message(msg);
        m.components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.label("skip role");
                    b.style(ButtonStyle::Secondary);
                    b.custom_id(SKIP)
                });
                a.create_button(|b| {
                    b.label("cancel");
                    b.style(ButtonStyle::Danger);
                    b.custom_id(CANCEL)
                })
            })
        }); 

        m.embed(|e| create_role_menu_embed(e, &roles, 0))
    }).await?;
    
    let guild = guild.unwrap();
    let mut reactions = guild.await_reactions(ctx)
        .author_id(msg.author.id)
        .guild_id(guild.id)
        .collect_limit(roles.len() as u32)
        .timeout(Duration::from_secs(60 * 5))
        .await
        .enumerate();

    let i = Arc::new(AtomicUsize::new(0));
    
    let (menu_listener_handle, menu_listener_registration) = AbortHandle::new_pair();
    let (abortable_reaction_listener, handle) = future::abortable(async {
        let i = i.clone();
        while let Some((_, reaction)) = reactions.next().await {
            let reaction = reaction.as_inner_ref();

            pool.create(guild.id, reaction.channel_id, reaction.message_id, reaction.emoji.clone(), roles[i.load(Ordering::Relaxed)]);
            
            rep.clone().edit(ctx, |m| {
                i.fetch_add(1, Ordering::Relaxed);
                let pos = i.load(Ordering::Relaxed);
                m.embed(|e| create_role_menu_embed(e, &roles, pos))
            }).await.unwrap(); 
    
            ctx.http.create_reaction(reaction.channel_id.0, reaction.message_id.0, &reaction.emoji).await.unwrap();

            if i.load(Ordering::Relaxed) == roles.len() {
                break;
            }
        }

        menu_listener_handle.abort();
    });

    let abortable_menu_listener = Abortable::new(async { 
        let cnt = i.clone();
        loop {
            let interaction_result = rep.await_component_interaction(ctx)
                .author_id(msg.author.id)
                .message_id(rep.id.0)
                .then(|a| async { 
                    if let Some(interaction) = a {
                        return match interaction.data.custom_id.parse().unwrap() {
                            CANCEL => {
                                interaction.create_interaction_response(ctx, |cir| {
                                    cir.kind(InteractionResponseType::DeferredUpdateMessage)
                                }).await.unwrap();

                                true
                            },
                            SKIP => {
                                cnt.fetch_add(1, Ordering::Relaxed);

                                let i = cnt.load(Ordering::Relaxed);
                                interaction.create_interaction_response(ctx, |cir| {
                                    cir.kind(InteractionResponseType::UpdateMessage)
                                        .interaction_response_data(|m| {
                                            m.create_embed(|e| create_role_menu_embed(e, &roles, i))
                                        })
                                }).await.unwrap();
                                
                                i == roles.len()
                            }
                            _ => false
                        }
                    }
                    false 
                });
            
            if interaction_result.await {
                break;
            }
        }

        handle.abort();
    }, menu_listener_registration);

    let _ = future::join(abortable_reaction_listener, abortable_menu_listener).await;

    rep.delete(ctx).await?;
    
    Ok(())
}
