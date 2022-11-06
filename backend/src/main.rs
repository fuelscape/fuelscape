use std::boxed::Box;
use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::str::FromStr;

use futures::executor::block_on;

use serde::Deserialize;
use serde::Serialize;

use fuel_crypto::SecretKey;
use fuel_gql_client::fuel_tx::ContractId;
use fuels::prelude::abigen;
use fuels::prelude::TxParameters;
use fuels_signers::provider::Provider;
use fuels_signers::wallet::DEFAULT_DERIVATION_PATH_PREFIX;
use fuels_signers::WalletUnlocked;
use fuels_types::bech32::Bech32Address;
use fuels_types::bech32::Bech32ContractId;

use poem::http::Method;
use poem::listener::TcpListener;
use poem::middleware::Cors;
use poem::web::Json;
use poem::web::Path;
use poem::EndpointExt;
use poem::Result;
use poem::Route;
use poem::Server;

use poem::get;
use poem::handler;
use poem::post;

use poem::error::BadGateway;
use poem::error::BadRequest;
use poem::error::InternalServerError;

const API_PORT: &str = "8080";
const NODE_URL: &str = "node-beta-1.fuel.network";
const WALLET_MNEMONIC: &str = "wet person force drum vicious milk afraid target treat verify faculty dilemma forget across congress visa hospital skull twenty sick ship tent limit survey";
const CONTRACT_ID: &str = "0xeadd5a23608fbde7c787590d1c6925a958fdb627e0f03c39edb06d65aa140d25";

abigen!(FuelScape, "../contract/out/debug/fuelscape-abi.json");

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let cors = Cors::new().allow_methods([Method::GET, Method::POST, Method::DELETE]);

    let app = Route::new()
        .at("/", get(version))
        .at("/locks/", post(create_lock).delete(delete_lock))
        .at("/items/", post(create_item).delete(delete_item))
        .at("/items/:wallet", get(list_items))
        .with(cors);

    let url = format!("127.0.0.1:{}", API_PORT);
    Server::new(TcpListener::bind(url)).run(app).await
}

#[derive(Serialize)]
struct Version {
    name: String,
    version: String,
}

#[handler]
fn version() -> Json<Version> {
    Json(Version{
        name: String::from("fuelscape"),
        version: String::from("0.1.0"),
    })
}


#[derive(Deserialize)]
struct CreateLockRequest {
    wallet: String,
}

#[derive(Serialize)]
struct CreateLockResponse {
    wallet: String,
    locked: bool,
    logs: Vec<String>,
}

#[handler]
async fn create_lock(req: Json<CreateLockRequest>) -> Result<Json<CreateLockResponse>> {
    let address = match Bech32Address::from_str(&req.wallet) {
        Ok(address) => address,
        Err(err) => return Err(BadRequest(err)),
    };

    let fuelscape = get_contract().await?;

    let lock = fuelscape
        .methods()
        .lock(address.into())
        .tx_params(TxParameters::new(Some(1), Some(1000000), None));
    let result = match block_on(lock.call()) {
        Ok(result) => result,
        Err(err) => return Err(InternalServerError(err)),
    };

    let res = CreateLockResponse {
        wallet: req.wallet.clone(),
        locked: true,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

    Ok(Json(res))
}

#[derive(Deserialize)]
struct DeleteLockRequest {
    wallet: String,
}

#[derive(Serialize)]
struct DeleteLockResponse {
    wallet: String,
    locked: bool,
    logs: Vec<String>,
}

#[handler]
async fn delete_lock(req: Json<DeleteLockRequest>) -> Result<Json<DeleteLockResponse>> {
    let address = match Bech32Address::from_str(&req.wallet) {
        Ok(address) => address,
        Err(err) => return Err(BadRequest(err)),
    };

    let fuelscape = get_contract().await?;

    let lock = fuelscape
        .methods()
        .unlock(address.into())
        .tx_params(TxParameters::new(Some(1), Some(1000000), None));
    let result = match block_on(lock.call()) {
        Ok(result) => result,
        Err(err) => return Err(InternalServerError(err)),
    };

    let res = DeleteLockResponse {
        wallet: req.wallet.clone(),
        locked: false,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

    Ok(Json(res))
}

#[derive(Deserialize)]
struct CreateItemRequest {
    wallet: String,
    item: u16,
    amount: u32,
}

#[derive(Serialize)]
struct CreateItemResponse {
    wallet: String,
    item: u16,
    balance: u32,
    logs: Vec<String>,
}

#[handler]
async fn create_item(req: Json<CreateItemRequest>) -> Result<Json<CreateItemResponse>> {
    let address = match Bech32Address::from_str(&req.wallet) {
        Ok(address) => address,
        Err(err) => return Err(BadRequest(err)),
    };

    let fuelscape = get_contract().await?;

    let give = fuelscape
        .methods()
        .give(address.into(), req.item, req.amount)
        .tx_params(TxParameters::new(Some(1), Some(1000000), None));
    let result = match block_on(give.call()) {
        Ok(result) => result,
        Err(err) => return Err(InternalServerError(err)),
    };

    let res = CreateItemResponse {
        wallet: req.wallet.clone(),
        item: req.item,
        balance: result.value,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

    Ok(Json(res))
}

#[derive(Deserialize)]
struct DeleteItemRequest {
    wallet: String,
    item: u16,
    amount: u32,
}

#[derive(Serialize)]
struct DeleteItemResponse {
    wallet: String,
    item: u16,
    balance: u32,
    logs: Vec<String>,
}

#[handler]
async fn delete_item(req: Json<DeleteItemRequest>) -> Result<Json<DeleteItemResponse>> {
    let address = match Bech32Address::from_str(&req.wallet) {
        Ok(address) => address,
        Err(err) => return Err(BadRequest(err)),
    };

    let fuelscape = get_contract().await?;

    let take = fuelscape
        .methods()
        .take(address.into(), req.item, req.amount)
        .tx_params(TxParameters::new(Some(1), Some(1000000), None));
    let result = match block_on(take.call()) {
        Ok(result) => result,
        Err(err) => return Err(InternalServerError(err)),
    };

    let res = DeleteItemResponse {
        wallet: req.wallet.clone(),
        item: req.item,
        balance: result.value,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

    Ok(Json(res))
}

#[derive(Serialize)]
struct ListItemsResponse {
    player: String,
    balances: HashMap<u16, u32>,
}

#[handler]
async fn list_items(Path(wallet): Path<String>) -> Result<Json<ListItemsResponse>> {
    let address = match Bech32Address::from_str(&wallet) {
        Ok(address) => address,
        Err(err) => return Err(BadRequest(err)),
    };

    let fuelscape = get_contract().await?;

    let view = fuelscape
        .methods()
        .view(address.clone().into())
        .tx_params(TxParameters::new(Some(1), Some(100000000), None));
    let result = match block_on(view.call()) {
        Ok(result) => result,
        Err(err) => return Err(InternalServerError(err)),
    };

    let entries = match fuelscape.logs_with_type::<Entry>(&result.receipts) {
        Ok(entries) => entries,
        Err(err) => return Err(InternalServerError(err)),
    };

    let balances = entries
        .iter()
        .map(|entry| (entry.item, entry.balance))
        .collect();

    let res = ListItemsResponse {
        player: wallet.clone(),
        balances,
    };

    Ok(Json(res))
}

async fn get_contract() -> Result<FuelScape> {
    let provider = match Provider::connect(NODE_URL).await {
        Ok(provider) => provider,
        Err(err) => return Err(BadGateway(err)),
    };

    let path = format!("{}/{}'/0/0", DEFAULT_DERIVATION_PATH_PREFIX, 0);
    let mnemonic: Vec<u8> = WALLET_MNEMONIC.bytes().collect();
    let phrase = match String::from_utf8(mnemonic) {
        Ok(phrase) => phrase,
        Err(err) => return Err(InternalServerError(err)),
    };

    let secret = match SecretKey::new_from_mnemonic_phrase_with_path(&phrase, &path) {
        Ok(secret) => secret,
        Err(err) => return Err(InternalServerError(err)),
    };
    let wallet = WalletUnlocked::new_from_private_key(secret, Some(provider));

    let address = match ContractId::from_str(CONTRACT_ID) {
        Ok(address) => address,
        Err(msg) => {
            return Err(InternalServerError(Error::new(
                ErrorKind::InvalidInput,
                msg,
            )))
        }
    };

    let bech32 = Bech32ContractId::from(address);
    let fuelscape = FuelScape::new(bech32.to_string(), wallet);

    return Ok(fuelscape);
}
