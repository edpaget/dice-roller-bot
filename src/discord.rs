use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::{Client as SerenityClient, Context, EventHandler, GatewayIntents, TypeMapKey},
};
use tokio::runtime::Handle;

use crate::{
    dynamodb::{make_client, DDBClient},
    repl::{REPLContext, REPL},
};

struct Handler;

impl TypeMapKey for DDBClient {
    type Value = DDBClient;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut data = ctx.data.write().await;
        let ddb_client = data.get_mut::<DDBClient>().unwrap();
        let handle = Handle::current();
        let repl_ctx = &REPLContext::new(msg.channel_id.to_string(), msg.author.name);
        let mut repl = REPL::new(&ddb_client.client, &handle);
        match repl.exec(repl_ctx, &msg.content) {
            Ok(eval_result) => {
                let response = format!("{}\n", eval_result);

                if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                    println!("Error sending message: {:?}", why);
                }
            }
            Err(_) => println!("Error parsing or evaluating AST"),
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let mut data = ctx.data.write().await;
        let ddb_client = make_client(false).await.expect("cannot start DDB client");
        data.insert::<DDBClient>(ddb_client);
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
pub async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = SerenityClient::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
