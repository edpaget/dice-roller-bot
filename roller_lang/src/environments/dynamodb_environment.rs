use crate::dynamodb::DDBClient;
use crate::types::{Context, Environment, Expression};
use std::collections::HashMap;

#[derive(Clone)]
pub struct DynamoDBEnvironment {
    client: DDBClient,
}

impl DynamoDBEnvironment {
    pub fn new(client: DDBClient) -> Self {
        DynamoDBEnvironment { client }
    }
}

impl Environment for DynamoDBEnvironment {
    async fn get<C: Context>(&self, ctx: C, var_name: &str) -> Option<Expression> {
        self.client
            .get_expression(&ctx.user_context_key(), &format!("var_name:{}", var_name))
            .await
            .ok()
    }

    async fn set<C: Context>(&mut self, ctx: C, var_name: &str, result: &Expression) {
        self.client
            .set_expression(
                &ctx.user_context_key(),
                &format!("var_name:{}", var_name),
                result,
            )
            .await
    }

    async fn print<C: Context>(&self, ctx: C) -> String {
        format!("dynamo-env:{}", ctx.user_context_key())
    }

    async fn closure<C: Context>(&self, ctx: C) -> Result<HashMap<String, Expression>, ()> {
        self.client.get_all_in_scope(&ctx.user_context_key()).await
    }
}

#[cfg(test)]
mod tests {

    use crate::dynamodb::{make_client, DDBClient};

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
    async fn make_env(client: &DDBClient) -> Result<DynamoDBEnvironment, Error> {
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
            .client
            .delete_table()
            .table_name("dice-roller-test")
            .send()
            .await;

        let _ = client
            .client
            .create_table()
            .table_name("dice-roller-test")
            .key_schema(pks)
            .key_schema(sks)
            .attribute_definitions(pk)
            .attribute_definitions(sk)
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await;

        Ok(DynamoDBEnvironment::new(client.clone()))
    }

    #[tokio::test]
    async fn test_save_read_dynamo() {
        let client = DDBClient::new(
            make_client(true).await.expect("failed to create client"),
            "dice-roller-test".to_string(),
        );
        let mut env = make_env(&client).await.expect("failed to create env");
        let ctx = &TestCtx;
        env.set(ctx, "test_value", &Expression::Integer(1)).await;
        assert_eq!(
            env.get(ctx, "test_value").await.unwrap(),
            Expression::Integer(1)
        )
    }
}
