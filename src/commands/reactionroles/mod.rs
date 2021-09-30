use serenity::framework::standard::macros::group;

mod setup;

use setup::*;

#[group]
#[prefixes("reactionrole", "rr")]
#[only_in(guilds)]
#[summary = "This command group contains commands that lets you manage reaction roles."]
#[description = "This command group contains commands that lets you manage reaction roles. The setup command is the default command so you can just type the reactionrole or rr to setup reaction roles."]
#[commands(setup)]
#[default_command(setup)]
#[required_permissions("ADMINISTRATOR")]
struct ReactionRoles;   