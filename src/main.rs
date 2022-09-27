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
use async_trait::async_trait;
use dotenv::dotenv;
use hex::FromHex;
use serde::Deserialize;
use std::{
    convert::TryFrom,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use url::Url;

const APTOS_CONFIG: &str = ".aptos/config.yaml";
const DEFAULT_TO_AMOUNT: u64 = 10_00000000_u64;

#[tokio::main]
async fn main() -> Result<()> {
    // setup env and log
    dotenv().ok();
    pretty_env_logger::init();

    let config = std::fs::read_to_string(APTOS_CONFIG)?;
    let config: Config = serde_yaml::from_str(&config)?;
    log::debug!("{:?}", &config);

    let rest_url = config.profiles.default.rest_url;
    let account = format!("0x{}", config.profiles.default.account);
    let private_key = &config.profiles.default.private_key[2..];

    // setup client
    let client = Client::new(Url::from_str(&rest_url)?);
    log::debug!("client\n{:?}\n", &client);

    // setup owner
    let address = AccountAddress::from_hex_literal(&account)?;
    let private_key = <[u8; 32]>::from_hex(private_key)?;
    let key = AccountKey::from_private_key(Ed25519PrivateKey::try_from(private_key.as_slice())?);
    let sequence_number = client.get_sequence_number(address).await?;
    let mut owner = LocalAccount::new(address, key, sequence_number);
    log::debug!("owner\n{:?}\n", &owner);
    log::info!("owner address: 0x{}", &owner.address());
    log::info!("owner sequence number: {}", sequence_number);

    // setup coin type and coin store
    let coin_type = format!("0x{}::island_coin::IslandCoin", &address);
    log::debug!("coin type\n{}\n", coin_type);
    let coin_store = format!("0x1::coin::CoinStore<{}>", &coin_type);
    log::debug!("coin store\n{}\n", coin_type);

    // check owner's IslandCoin balance
    // let to_address = std::env::var("TO_ADDRESS").unwrap_or(format!("0x{}", address));
    let mut args = std::env::args();
    args.next();
    let to_address = args.next().unwrap_or(account);
    let to_amount = match args.next() {
        Some(amount) => amount.parse().unwrap_or(DEFAULT_TO_AMOUNT),
        None => DEFAULT_TO_AMOUNT,
    };
    log::info!("to address: {}", to_address);
    let to_address = AccountAddress::from_hex_literal(&to_address).unwrap_or(address);
    let balance = client
        .get_island_coin_balance(to_address, &coin_store)
        .await?;
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

    // check owner's IslandCoin balance
    let balance = client
        .get_island_coin_balance(to_address, &coin_store)
        .await?;
    log::info!("balance after : {}", &balance);

    Ok(())
}

#[async_trait]
trait IslandCoinClient {
    async fn get_island_coin_balance(
        &self,
        address: AccountAddress,
        coin_store: &str,
    ) -> Result<u64>;

    async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64>;
}

#[async_trait]
impl IslandCoinClient for Client {
    async fn get_island_coin_balance(
        &self,
        address: AccountAddress,
        coin_store: &str,
    ) -> Result<u64> {
        let resp = self.get_account_resource(address, coin_store).await?;
        resp.and_then(|resource| {
            if let Some(res) = resource {
                log::debug!("coin resource:\n{:?}\n", res);
                Ok(serde_json::from_value::<Balance>(res.data)?)
            } else {
                Err(anyhow!("No CoinStore resource under account"))
            }
        })
        .map(|resp| resp.inner().get())
    }

    async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        let account_resp = self
            .get_account(address)
            .await
            .map_err(|err| anyhow!(err.to_string()))?;
        let account = account_resp.inner();
        Ok(account.sequence_number)
    }
}

#[derive(Deserialize, Debug)]
struct Profile {
    private_key: String,
    // public_key: String,
    account: String,
    rest_url: String,
    // faucet_url: String,
}

#[derive(Deserialize, Debug)]
struct Profiles {
    default: Profile,
}

#[derive(Deserialize, Debug)]
struct Config {
    profiles: Profiles,
}
