use anyhow::Result;
use aptos_playground::{AptosConfig, ExtentedAptosClient};
use aptos_sdk::bcs;
use aptos_sdk::move_types::{identifier::Identifier, language_storage::ModuleId};
use aptos_sdk::rest_client::Client;
use aptos_sdk::transaction_builder::TransactionBuilder;
use aptos_sdk::types::{
    account_address::{create_resource_address, AccountAddress},
    chain_id::ChainId,
    transaction::{EntryFunction, TransactionPayload},
    AccountKey, LocalAccount,
};
use dotenv::dotenv;
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    // setup env and log
    dotenv().ok();
    pretty_env_logger::init();

    let default_profile = AptosConfig::load_profile("mainnet")?;
    log::debug!("{:?}", &default_profile);

    let rest_url = default_profile
        .rest_url
        .expect("'rest_url' field not found");
    let address = default_profile.account.expect("'account' field not found");
    let private_key = default_profile
        .private_key
        .expect("'private_key' field not found");

    // setup client
    let client = Client::new(Url::from_str(&rest_url)?);
    log::debug!("client\n{:?}\n", &client);

    let table_handle = AccountAddress::from_hex_literal(
        "0x64997f0422516f96db479fa4789d095c8612bd412d44e592f5a325751c9fb36f",
    )?;
    let key_type = "0x1::string::String";
    let value_type = "0x3::token::CollectionData";
    let key = "Fake Aptos Polar Bears 1";
    let collection_data = client
        .get_table_item(table_handle, key_type, value_type, key)
        .await?
        .inner()
        .clone();
    log::info!("{:?}", collection_data);

    let resource_account = AccountAddress::from_hex_literal(
        "0xcacda9e05cb789634ac5d430176035b44c6bb28baad3f6433f4d3f3c4578dfa2",
    )?;
    let resource_type = "0x3::token::Collections";
    let collections_metadata = client
        .get_account_resource(resource_account, resource_type)
        .await?
        .inner()
        .clone();
    log::info!("{:?}", collections_metadata);

    Ok(())
}
