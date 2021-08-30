mod reactionroles;

use std::collections::HashSet;

pub use reactionroles::*;
use serenity::{client::Context, framework::standard::{Args, CommandGroup, CommandResult, HelpOptions, help_commands, macros::help}, model::{channel::Message, id::UserId}};

#[help]
#[individual_command_tip ="Hello!"]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}