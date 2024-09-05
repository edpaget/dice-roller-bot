use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use crate::{
    environments::hash_map_environment::HashMapEnvironment,
    repl::{REPLContext, REPL},
};

struct Handler;

impl TypeMapKey for REPL<HashMapEnvironment> {
    type Value = REPL<HashMapEnvironment>;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut data = ctx.data.write().await;
        let repl_ctx = &REPLContext::new(msg.channel_id.to_string(), msg.author.name);
        let repl = data.get_mut::<REPL<HashMapEnvironment>>().unwrap();
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
        data.insert::<REPL<HashMapEnvironment>>(REPL::default());
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
pub async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
