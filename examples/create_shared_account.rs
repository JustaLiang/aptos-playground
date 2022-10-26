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

    // setup owner
    let key = AccountKey::from_private_key(private_key);
    let sequence_number = client.get_sequence_number(address).await?;
    let mut owner = LocalAccount::new(address, key, sequence_number);
    log::debug!("owner\n{:?}\n", &owner);
    log::info!("owner address: {}", owner.address().to_hex_literal());
    log::info!("owner sequence number: {}", sequence_number);

    // send transaction
    let chain_id = client
        .get_index()
        .await
        .expect("Failed to fetch chain ID")
        .inner()
        .chain_id;
    let estimated_gas_price = client.estimate_gas_price().await?.inner().gas_estimate;

    let seed = "AptosPolarBearsRoyalty";
    let seed_slice = seed.as_bytes();
    let seed = bcs::to_bytes(&seed)?;

    let addresses = vec![
        AccountAddress::from_hex_literal(
            "0xb4265f4e7c948d41a207e3f4f41b671619a6c37c263050a4259806983bb81fa",
        )?,
        AccountAddress::from_hex_literal(
            "0x9fc70369457ec4e09d102ce78bfa42eda002ea2c83d87245a898aae6b27de881",
        )?,
        AccountAddress::from_hex_literal(
            "0x2e83741b47b146c60d9e46244a85e1060f78c9ded73981a29aae7d711740af0c",
        )?,
    ];
    let addresses = bcs::to_bytes(&addresses)?;

    let shares = vec![92u64, 4u64, 4u64];
    let shares = bcs::to_bytes(&shares)?;

    let transaction = TransactionBuilder::new(
        TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(owner.address(), Identifier::new("shared_account")?),
            Identifier::new("initialize")?,
            vec![],
            vec![seed, addresses, shares],
        )),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 300,
        ChainId::new(chain_id),
    )
    .sender(owner.address())
    .sequence_number(owner.sequence_number())
    .max_gas_amount(10_000)
    .gas_unit_price(estimated_gas_price);
    let signed_txn = owner.sign_with_transaction_builder(transaction);
    log::debug!("transaction\n{:?}\n", &signed_txn);
    let pending_txn = client.submit(&signed_txn).await?;
    let transaction = client.wait_for_transaction(pending_txn.inner()).await?;
    log::debug!("transaction\n{:?}\n", &transaction);

    log::info!(
        "shared account: {}",
        create_resource_address(owner.address(), seed_slice).to_hex_literal()
    );

    Ok(())
}
