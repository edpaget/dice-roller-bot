use std::env;

use serenity::prelude::{Client as SerenityClient, GatewayIntents};

#[tokio::main]
pub async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = SerenityClient::builder(&token, intents)
        .event_handler(roller_lang::discord::Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
