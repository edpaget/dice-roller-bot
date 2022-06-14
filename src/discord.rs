use std::env;

use rand::SeedableRng;
use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*, async_trait,
};

use crate::{environments::hash_map_environment::HashMapEnvironment, types::Visitor};
use crate::parser::command;
use crate::eval::EvalVisitor;

struct Handler;

impl TypeMapKey for HashMapEnvironment {
    type Value = HashMapEnvironment;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let mut data = ctx.data.write().await;
        let env = data.get_mut::<HashMapEnvironment>().unwrap();
        if let Ok((_, ast)) = command(&msg.content) {
            let mut visitor = EvalVisitor::new(
                &mut rng,
                env,
                &msg.author.name,
            );
            let response = format!("{}\n", visitor.visit_statement(Box::new(ast)).unwrap());

            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let mut data = ctx.data.write().await;
        data.insert::<HashMapEnvironment>(HashMapEnvironment::new());
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
pub async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
