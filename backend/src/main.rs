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
use fuels_signers::provider::Provider;
use fuels_signers::wallet::DEFAULT_DERIVATION_PATH_PREFIX;
use fuels_signers::WalletUnlocked;
use fuels_types::bech32::Bech32Address;

use poem::listener::TcpListener;
use poem::web::Json;
use poem::web::Path;
use poem::Result;
use poem::Route;
use poem::Server;

use poem::get;
use poem::delete;
use poem::handler;
use poem::post;

use poem::error::BadGateway;
use poem::error::BadRequest;
use poem::error::InternalServerError;

const API_PORT: &str = "8080";
const NODE_URL: &str = "node-beta-1.fuel.network";
const WALLET_MNEMONIC: &str = "wet person force drum vicious milk afraid target treat verify faculty dilemma forget across congress visa hospital skull twenty sick ship tent limit survey";
const CONTRACT_ID: &str = "0xf473a5437d25e389bf6244f777e9f1909b3aa69d58393a16ff8962543378014d";

use fuels::prelude::abigen;
abigen!(FuelScape,"../contract/out/debug/fuelscape-abi.json");

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .at("/locks/", post(create_lock))
        .at("/locks/", delete(delete_lock))
        .at("/items/", post(create_item))
        .at("/items/", delete(delete_item))
        .at("/items/:wallet", get(list_items));

    let url = format!("127.0.0.1:{}", API_PORT);
    Server::new(TcpListener::bind(url)).run(app).await
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

    let lock = fuelscape.methods().lock(address.into());
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

    let lock = fuelscape.methods().unlock(address.into());
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
    item: u64,
    amount: u64,
}

#[derive(Serialize)]
struct CreateItemResponse {
    wallet: String,
    item: u64,
    amount: u64,
    logs: Vec<String>,
}

#[handler]
async fn create_item(req: Json<CreateItemRequest>) -> Result<Json<CreateItemResponse>> {
    let address = match Bech32Address::from_str(&req.wallet) {
        Ok(address) => address,
        Err(err) => return Err(BadRequest(err)),
    };

    let fuelscape = get_contract().await?;
    
    let give = fuelscape.methods().give(address.into(), req.item, req.amount);
    let result = match block_on(give.call()) {
            Ok(result) => result,
            Err(err) => return Err(InternalServerError(err)),
    };

    let res = CreateItemResponse {
        wallet: req.wallet.clone(),
        item: req.item,
        amount: result.value,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

    Ok(Json(res))
}

#[derive(Deserialize)]
struct DeleteItemRequest {
    wallet: String,
    item: u64,
    amount: u64,
}

#[derive(Serialize)]
struct DeleteItemResponse {
    wallet: String,
    item: u64,
    amount: u64,
    logs: Vec<String>,
}

#[handler]
async fn delete_item(req: Json<DeleteItemRequest>) -> Result<Json<DeleteItemResponse>> {
    let address = match Bech32Address::from_str(&req.wallet) {
        Ok(address) => address,
        Err(err) => return Err(BadRequest(err)),
    };


    let fuelscape = get_contract().await?;
    
    let take = fuelscape.methods().take(address.into(), req.item, req.amount);
    let result = match block_on(take.call()) {
            Ok(result) => result,
            Err(err) => return Err(InternalServerError(err)),
    };

    let res = DeleteItemResponse {
        wallet: req.wallet.clone(),
        item: req.item,
        amount: result.value,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

    Ok(Json(res))
}

#[derive(Serialize)]
struct ListItemsResponse {
    player: String,
    items: HashMap<u64, u64>,
}

#[handler]
async fn list_items(Path(wallet): Path<String>) -> Result<Json<ListItemsResponse>> {
    let address = match Bech32Address::from_str(&wallet) {
        Ok(address) => address,
        Err(err) => return Err(BadRequest(err)),
    };

    let fuelscape = get_contract().await?;
    
    let mut items = HashMap::new();
    for item in 0..32768 {
        let view = fuelscape.methods().view(address.clone().into(), item);
        let result = match block_on(view.call()) {
                Ok(result) => result,
                Err(err) => return Err(InternalServerError(err)),
        };
        items.insert(item, result.value);
    }


    let res = ListItemsResponse{
        player: wallet.clone(),
        items: items,
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
        Err(msg) => return Err(InternalServerError(Error::new(
            ErrorKind::InvalidInput,
            msg,
        ))),
    };

    let fuelscape = FuelScape::new(address.into(), wallet);

    return Ok(fuelscape);
}
