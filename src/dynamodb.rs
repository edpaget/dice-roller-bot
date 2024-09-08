use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{Client, Error};

const LOCALSTACK_ENDPOINT: &str = "http://localhost:4566/";

pub async fn make_client(use_localstack: bool) -> Result<Client, Error> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let mut config = aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
    if use_localstack {
        config = config.endpoint_url(LOCALSTACK_ENDPOINT);
    }
    let config = config.load().await;
    Ok(Client::new(&config))
}