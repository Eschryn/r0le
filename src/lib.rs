use std::sync::Arc;

use futures::{future::{self, AbortHandle, Abortable}, FutureExt};
use serenity::{async_trait, builder::CreateMessage, framework::standard::CommandResult, model::{interactions::message_component::MessageComponentInteraction, prelude::*}, prelude::*};


#[async_trait]
pub trait WorkerMenu {
    fn create_ui<'a, 'b>(&self, m: &'a mut CreateMessage<'b>) -> &'a mut CreateMessage<'b>;

    async fn open(&self, ctx: &Context, reference_message: &Message) -> CommandResult {
        let rep = reference_message.channel_id.send_message(ctx, |m| {
            m.reference_message(reference_message);
            self.create_ui(m)
        }).await?;

        let (menu_listener_handle, menu_listener_registration) = AbortHandle::new_pair();
        let (abortable_reaction_listener, handle) = future::abortable(async {
            self.process(&ctx, &rep).await;
            menu_listener_handle.abort();
        });
    
        let abortable_menu_listener = Abortable::new(async {
            loop {
                let interaction_result = rep.await_component_interaction(ctx)
                    .author_id(reference_message.author.id)
                    .message_id(rep.id.0)
                    .then(|a| async { 
                        if let Some(interaction) = a {
                            self.ui_event(&ctx, &interaction).await
                        } else {
                            false 
                        }
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

    async fn process(&self, ctx: &Context, menu: &Message);
    async fn ui_event(&self, ctx: &Context, interaction: &Arc<MessageComponentInteraction>) -> bool;
}