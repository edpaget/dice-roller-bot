use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::{Context, EventHandler, TypeMapKey},
};

use crate::{
    dynamodb::{make_client, DDBClient},
    environments::dynamodb_environment::DynamoDBEnvironment,
    repl::{REPLContext, REPL},
};

pub struct Handler;

impl TypeMapKey for REPL<DynamoDBEnvironment> {
    type Value = REPL<DynamoDBEnvironment>;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.content.starts_with('!') {
            return;
        }

        let mut data = ctx.data.write().await;
        let repl = data.get_mut::<REPL<DynamoDBEnvironment>>().unwrap();
        let repl_ctx = &REPLContext::new(msg.channel_id.to_string(), msg.author.name);
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
        let ddb_client = DDBClient::with_default_table(
            make_client(false).await.expect("cannot start DDB client"),
        );
        let repl = REPL::new(ddb_client);
        data.insert::<REPL<DynamoDBEnvironment>>(repl);
        println!("{} is connected!", ready.user.name);
    }
}
