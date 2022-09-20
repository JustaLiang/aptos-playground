use anyhow::{anyhow, Result};
use aptos_sdk::{
    bcs,
    crypto::ed25519::Ed25519PrivateKey,
    move_types::{
        identifier::Identifier,
        language_storage::{ModuleId, TypeTag},
    },
    rest_client::{aptos::Balance, Client},
    transaction_builder::TransactionBuilder,
    types::{
        account_address::AccountAddress,
        chain_id::ChainId,
        transaction::{EntryFunction, TransactionPayload},
        AccountKey, LocalAccount,
    },
};
use dotenv::dotenv;
use hex::FromHex;
use std::{
    convert::TryFrom,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    // setup env and log
    dotenv().ok();
    pretty_env_logger::init();
    let node_url = std::env::var("APTOS_NODE_URL").expect("APTOS_NODE_URL not set");
    let account_address = std::env::var("ACCOUNT_ADDRESS").expect("ACCOUNT_ADDRESS not set");
    let private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let sequence_number = std::env::var("SEQUENCE_NUMBER")
        .expect("SEQUENCE_NUMBER not set")
        .parse::<u64>()
        .expect("SEQUENCE_NUMBER not u64");
    log::info!("load env done");

    // setup client
    let client = Client::new(Url::from_str(&node_url)?);
    log::debug!("client\n{:?}\n", &client);

    // setup owner
    let address = AccountAddress::from_hex_literal(&account_address)?;
    let private_key = <[u8; 32]>::from_hex(&private_key)?;
    let key = AccountKey::from_private_key(Ed25519PrivateKey::try_from(&private_key as &[u8])?);
    let mut owner = LocalAccount::new(address, key, sequence_number);
    log::debug!("owner\n{:?}\n", &owner);
    log::info!("owner address: {}", &owner.address());

    // setup coin type and coin store
    let coin_type = format!("0x{}::island_coin::IslandCoin", &address);
    log::debug!("coin type\n{}\n", coin_type);
    let coin_store = format!("0x1::coin::CoinStore<{}>", &coin_type);
    log::debug!("coin store\n{}\n", coin_type);

    // check owner's IslandCoin balance
    let to_address = std::env::var("ACCOUNT_ADDRESS").unwrap_or(format!("0x{}", address));
    let to_address = AccountAddress::from_hex_literal(&to_address)?;
    let balance = get_island_coin_balance(&client, to_address, &coin_store)
        .await
        .unwrap_or(0);
    log::info!("balance before: {}", &balance);

    // send transaction
    let chain_id = client
        .get_index()
        .await
        .expect("Failed to fetch chain ID")
        .inner()
        .chain_id;
    let transaction = TransactionBuilder::new(
        TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(AccountAddress::ONE, Identifier::new("managed_coin")?),
            Identifier::new("mint")?,
            vec![TypeTag::from_str(&coin_type)?],
            vec![
                bcs::to_bytes(&to_address)?,
                bcs::to_bytes(&1_000_000_000_u64)?,
            ],
        )),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 300,
        ChainId::new(chain_id),
    )
    .sender(owner.address())
    .sequence_number(owner.sequence_number())
    .max_gas_amount(10_000)
    .gas_unit_price(1);
    let signed_txn = owner.sign_with_transaction_builder(transaction);
    log::debug!("transaction\n{:?}\n", &signed_txn);
    let pending_txn = client.submit(&signed_txn).await?;
    let transaction = client.wait_for_transaction(pending_txn.inner()).await?;
    log::debug!("transaction\n{:?}\n", &transaction);

    // check owner's IslandCoin balance
    let balance = get_island_coin_balance(&client, to_address, &coin_store)
        .await
        .unwrap_or(0);
    log::info!("balance after: {}", &balance);

    Ok(())
}

// if no resource return 0
async fn get_island_coin_balance(
    client: &Client,
    address: AccountAddress,
    coin_store: &str,
) -> Result<u64> {
    let resp = client.get_account_resource(address, coin_store).await?;
    resp.and_then(|resource| {
        if let Some(res) = resource {
            Ok(serde_json::from_value::<Balance>(res.data)?)
        } else {
            Err(anyhow!("No resource unde account"))
        }
    })
    .map(|balance| balance.inner().get())
}
