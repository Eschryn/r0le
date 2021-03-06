mod commands;
mod event_handler;
mod data;
mod opts;

use clap::Clap;

use data::RedisReactionRoleStore;
use serenity::{client::bridge::gateway::GatewayIntents, framework::standard::StandardFramework, prelude::*};

#[tokio::main]
async fn main() {
    let opts = opts::Opts::parse();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) 
        .group(&commands::REACTIONROLES_GROUP)
        .help(&commands::MY_HELP);

    let token = opts.token;
    let application_id = opts.application_id;
    let mut client = Client::builder(token)
        .event_handler(event_handler::Handler(application_id))
        .application_id(application_id)
        .framework(framework)
        .intents(GatewayIntents::all())
        .await
        .expect("Error creating client");

        
    {
        let store = RedisReactionRoleStore::connect(opts.redis_url.unwrap_or("redis://127.0.0.1".to_string())).await.unwrap();
        let mut data = client.data.write().await;

        data.insert::<RedisReactionRoleStore>(store)
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
