use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::{Context, EventHandler, TypeMapKey},
};

use crate::{
    environments::hash_map_environment::HashMapEnvironment,
    repl::{REPLContext, REPL},
};

pub struct Handler;

impl TypeMapKey for REPL<HashMapEnvironment> {
    type Value = REPL<HashMapEnvironment>;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.content.starts_with('!') {
            return;
        }

        let mut data = ctx.data.write().await;
        let repl = data.get_mut::<REPL<HashMapEnvironment>>().unwrap();
        let repl_ctx = &REPLContext::new(msg.channel_id.to_string(), msg.author.name.clone());
        let response = match repl.exec(repl_ctx, &msg.content).await {
            Ok(eval_result) => format!("{}\n", eval_result),
            Err(err) => {
                println!("Error: {} parsing or evaluating msg: {}", err, &msg.content);
                format!("{}\n", err)
            }
        };
        if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
            println!("Error sending message: {:?}", why);
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let mut data = ctx.data.write().await;
        let repl = REPL::<HashMapEnvironment>::default();
        data.insert::<REPL<HashMapEnvironment>>(repl);
        println!("{} is connected!", ready.user.name);

        if env::var("USE_DYNAMODB").is_ok() {
            println!("Warning: USE_DYNAMODB is set but DynamoDB support is disabled in this build");
        }
    }
}
