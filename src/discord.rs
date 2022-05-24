use std::env;

use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use crate::{environments::hash_map_environment::HashMapEnvironment, types::Visitor};
use crate::parser::command;
use crate::eval::EvalVisitor;

struct Handler;

impl TypeMapKey for HashMapEnvironment {
    type Value = HashMapEnvironment;
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        let mut rng = rand::thread_rng();
        let mut data = ctx.data.write();
        let env = data.get_mut::<HashMapEnvironment>().unwrap();
        if let Ok((_, ast)) = command(&msg.content) {
            let mut visitor = EvalVisitor{
                rng: &mut rng,
                env,
                user: &msg.author.name,
            };
            let response = format!("{}\n", visitor.visit_statement(Box::new(ast)).unwrap());

            if let Err(why) = msg.channel_id.say(&ctx.http, response) {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        let mut data = ctx.data.write();
        data.insert::<HashMapEnvironment>(HashMapEnvironment::new());
        println!("{} is connected!", ready.user.name);
    }
}

pub fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
