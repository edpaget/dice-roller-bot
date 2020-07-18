use std::env;

use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use crate::parser::roll;
use crate::eval::eval;


struct Handler;

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        let mut rng = rand::thread_rng();
        if let Ok((_, ast)) = roll(&msg.content) {
            let response = format!("{}\n", eval(&mut rng, Box::new(ast)));

            if let Err(why) = msg.channel_id.say(&ctx.http, response) {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
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
