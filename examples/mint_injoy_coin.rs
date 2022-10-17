use anyhow::Result;
use aptos_playground::{AptosConfig, ExtentedAptosClient};
use aptos_sdk::bcs;
use aptos_sdk::move_types::{
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use aptos_sdk::rest_client::Client;
use aptos_sdk::transaction_builder::TransactionBuilder;
use aptos_sdk::types::{
    account_address::AccountAddress,
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

const DEFAULT_TO_AMOUNT: u64 = 10_00000000_u64;

#[tokio::main]
async fn main() -> Result<()> {
    // setup env and log
    dotenv().ok();
    pretty_env_logger::init();

    let default_profile = AptosConfig::load_profile("default")?;
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

    // setup coin type
    let coin_type = format!("0x{}::injoy_coin::InJoyCoin", &address);
    log::debug!("coin type\n{}\n", coin_type);

    // check owner's InJoyCoin balance
    let mut args = std::env::args();
    args.next();
    let to_address = if let Some(addr) = args.next() {
        AccountAddress::from_hex_literal(&addr)?
    } else {
        address
    };
    let to_amount = match args.next() {
        Some(amount) => amount.parse().unwrap_or(DEFAULT_TO_AMOUNT),
        None => DEFAULT_TO_AMOUNT,
    };
    log::info!("to address: 0x{}", to_address);
    let balance = client
        .get_account_balance_bcs(address, &coin_type)
        .await?
        .into_inner();
    log::info!("balance before: {}", &balance);

    // send transaction
    let chain_id = client
        .get_index()
        .await
        .expect("Failed to fetch chain ID")
        .inner()
        .chain_id;
    let estimated_gas_price = client.estimate_gas_price().await?.inner().gas_estimate;

    let transaction = TransactionBuilder::new(
        TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(AccountAddress::ONE, Identifier::new("managed_coin")?),
            Identifier::new("mint")?,
            vec![TypeTag::from_str(&coin_type)?],
            vec![bcs::to_bytes(&to_address)?, bcs::to_bytes(&to_amount)?],
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

    // check owner's InJoyCoin balance
    let balance = client
        .get_account_balance_bcs(address, &coin_type)
        .await?
        .into_inner();
    log::info!("balance after : {}", &balance);

    Ok(())
}
