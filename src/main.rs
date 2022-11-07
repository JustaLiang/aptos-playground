use anyhow::Result;
use aptos_playground::AptosConfig;
use aptos_sdk::rest_client::Client;
use aptos_sdk::types::account_address::AccountAddress;
use dotenv::dotenv;
use std::str::FromStr;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    // setup env and log
    dotenv().ok();
    pretty_env_logger::init();

    let profile = AptosConfig::load_profile("mainnet")?;
    log::debug!("{:?}", &profile);

    let rest_url = profile.rest_url.expect("'rest_url' field not found");

    // setup client
    let rest_url = Url::from_str(&rest_url)?;
    let client = Client::new(rest_url);
    log::debug!("client\n{:?}\n", &client);

    let collection_configs_handle = AccountAddress::from_hex_literal(
        "0x98d2b9903a3236185515b3ce8c6fe1171fdad3de9a03b4c63223c87ee752e1c1",
    )?;
    let spw_handle = client.get_table_item(
        collection_configs_handle,
        "0x1::string::String",
        "0x4b8cec33043700c2e159b55d39dff908c28f21ebaf0d64b0539a465721021a3a::candy_machine_v2::CollectionConfig",
        "Aptos Yetis"
    ).await?.inner().get("supply_per_wl").unwrap().get("handle").unwrap().to_string();
    let spw_handle_str = &spw_handle[1..spw_handle.len() - 1];
    log::info!("spw_handle: {}", spw_handle_str);

    let spw_handle = AccountAddress::from_hex_literal(spw_handle_str)?;

    let whitelist_user = AccountAddress::from_hex_literal(
        "0x3a8be6e9603996bd13e8ade795fb62035b4799fa82427bc0418174629ed7fdb5",
    )?;
    let whitelist_supply = client
        .get_table_item(spw_handle, "address", "u64", whitelist_user)
        .await?
        .into_inner();
    log::info!("{:?}", whitelist_supply);
    Ok(())
}
