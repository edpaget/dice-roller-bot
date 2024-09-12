use crate::types::{Context, Environment, Expression};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use serde_dynamo::aws_sdk_dynamodb_1::{from_item, to_item};

#[derive(Clone)]
pub struct DynamoDBEnvironment<'a> {
    client: &'a Client,
    table_name: String,
}

const DEFAULT_TABLE_NAME: &str = "dice-roller-bot";

impl<'a> DynamoDBEnvironment<'a> {
    pub fn new(client: &'a Client, table_name: String) -> Self {
        DynamoDBEnvironment { client, table_name }
    }

    pub fn with_default_table(client: &'a Client) -> Self {
        DynamoDBEnvironment::new(client, DEFAULT_TABLE_NAME.to_string())
    }
}

impl<'a> Environment for DynamoDBEnvironment<'a> {
    async fn get<C: Context>(&self, ctx: C, var_name: &str) -> Option<Expression> {
        let response = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", AttributeValue::S(ctx.user_context_key()))
            .key("sk", AttributeValue::S(format!("var_name:{}", var_name)))
            .send()
            .await;

        match response {
            Ok(res) => match from_item(res.item()?.clone()) {
                Ok(expr) => Some(expr),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    async fn set<C: Context>(&mut self, ctx: C, var_name: &str, result: &Expression) {
        if let Ok(item) = to_item(result) {
            let _ = self
                .client
                .put_item()
                .table_name(&self.table_name)
                .set_item(Some(item))
                .item("pk", AttributeValue::S(ctx.user_context_key()))
                .item("sk", AttributeValue::S(format!("var_name:{}", var_name)))
                .send()
                .await;
        }
    }

    async fn print<C: Context>(&self, ctx: C) -> String {
        format!("dynamo-env:{}", ctx.user_context_key())
    }
}

#[cfg(test)]
mod tests {

    use crate::dynamodb::make_client;

    use super::*;
    use aws_sdk_dynamodb::types::{
        AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ScalarAttributeType,
    };
    use aws_sdk_dynamodb::Error;

    struct TestCtx;

    impl Context for &TestCtx {
        fn user_context_key(&self) -> String {
            format!("scope:{}#scope_type:user#user:{}", "test", "test_user")
        }

        fn global_context_key(&self) -> String {
            format!("scope:{}#scope_type:user#user:{}", "test", "global")
        }
    }

    #[allow(clippy::result_large_err)]
    async fn make_env(client: &Client) -> Result<DynamoDBEnvironment, Error> {
        let pk = AttributeDefinition::builder()
            .attribute_name("pk")
            .attribute_type(ScalarAttributeType::S)
            .build()?;

        let sk = AttributeDefinition::builder()
            .attribute_name("sk")
            .attribute_type(ScalarAttributeType::S)
            .build()?;

        let pks = KeySchemaElement::builder()
            .attribute_name("pk")
            .key_type(KeyType::Hash)
            .build()?;

        let sks = KeySchemaElement::builder()
            .attribute_name("sk")
            .key_type(KeyType::Range)
            .build()?;

        let _ = client
            .delete_table()
            .table_name("dice-roller-test")
            .send()
            .await;

        let _ = client
            .create_table()
            .table_name("dice-roller-test")
            .key_schema(pks)
            .key_schema(sks)
            .attribute_definitions(pk)
            .attribute_definitions(sk)
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await;

        Ok(DynamoDBEnvironment::new(
            client,
            "dice-roller-test".to_string(),
        ))
    }

    #[tokio::test]
    async fn test_save_read_dynamo() {
        let client = make_client(true)
            .await
            .expect("failed to create client")
            .client;
        let mut env = make_env(&client).await.expect("failed to create env");
        let ctx = &TestCtx;
        env.set(ctx, "test_value", &Expression::Integer(1)).await;
        assert_eq!(
            env.get(ctx, "test_value").await.unwrap(),
            Expression::Integer(1)
        )
    }
}
