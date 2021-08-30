use serenity::framework::standard::macros::group;

mod setup;

use setup::*;

#[group]
#[prefixes("reactionrole", "rr")]
#[commands(setup)]
#[default_command(setup)]
struct ReactionRoles;