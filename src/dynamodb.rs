use crate::types::Expression;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error};
use serde_dynamo::aws_sdk_dynamodb_1::{from_item, to_item};
use std::collections::HashMap;

const LOCALSTACK_ENDPOINT: &str = "http://localhost:4566/";
const DEFAULT_TABLE_NAME: &str = "dice-roller-bot";

#[derive(Debug, Clone)]
pub struct DDBClient {
    pub client: Client,
    table_name: String,
}

impl DDBClient {
    pub fn new(client: Client, table_name: String) -> Self {
        DDBClient { client, table_name }
    }

    pub fn with_default_table(client: Client) -> Self {
        DDBClient::new(client, DEFAULT_TABLE_NAME.to_string())
    }

    pub async fn get_expression(&self, pk: &str, sk: &str) -> Result<Expression, ()> {
        match self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", AttributeValue::S(pk.to_string()))
            .key("sk", AttributeValue::S(sk.to_string()))
            .send()
            .await
        {
            Ok(res) => match from_item(res.item().ok_or(())?.clone()) {
                Ok(expr) => Ok(expr),
                Err(_) => Err(()),
            },
            Err(_) => Err(()),
        }
    }

    pub async fn set_expression(&self, pk: &str, sk: &str, expr: &Expression) {
        if let Ok(item) = to_item(expr) {
            let _ = self
                .client
                .put_item()
                .table_name(&self.table_name)
                .set_item(Some(item))
                .item("pk", AttributeValue::S(pk.to_string()))
                .item("sk", AttributeValue::S(sk.to_string()))
                .send()
                .await;
        }
    }

    pub async fn get_all_in_scope(&self, pk: &str) -> Result<HashMap<String, Expression>, ()> {
        let response = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("#pk = :pk")
            .expression_attribute_names("#pk", "pk")
            .expression_attribute_values(":pk", AttributeValue::S(pk.to_string()))
            .send()
            .await;

        match response {
            Ok(res) => {
                let mut new_env = HashMap::new();
                for item in res.items() {
                    match item.get("sk") {
                        Some(sk) => match from_item(item.clone()) {
                            Ok(expr) => {
                                new_env.insert(sk.as_s().unwrap().to_string(), expr);
                            }
                            Err(_) => return Err(()),
                        },
                        None => return Err(()),
                    }
                }
                Ok(new_env)
            }
            Err(_) => Err(()),
        }
    }
}

pub async fn make_client(use_localstack: bool) -> Result<Client, Error> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let mut config = aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
    if use_localstack {
        config = config.endpoint_url(LOCALSTACK_ENDPOINT);
    }
    let config = config.load().await;
    Ok(Client::new(&config))
}
