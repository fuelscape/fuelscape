#[macro_use]
extern crate lazy_static;

use std::boxed::Box;
use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::str::FromStr;
use std::sync::Mutex;

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
use poem::Result;
use poem::Route;
use poem::Server;

use poem::delete;
use poem::handler;
use poem::post;

use poem::error::BadGateway;
use poem::error::BadRequest;
use poem::error::Conflict;
use poem::error::InternalServerError;
use poem::error::NotFound;

const API_PORT: &str = "8080";
const NODE_URL: &str = "node-beta-1.fuel.network";
const WALLET_MNEMONIC: &str = "wet person force drum vicious milk afraid target treat verify faculty dilemma forget across congress visa hospital skull twenty sick ship tent limit survey";
const CONTRACT_ID: &str = "0x20696b1bae562519ad0d23a249429f3893938fbe7c152401d8c4e05cbc24b2fa";

use fuels::prelude::abigen;
abigen!(FuelScape,"../contract/out/debug/fuelscape-abi.json");

lazy_static! {
    static ref WALLET_LOOKUP: Mutex<HashMap<String, Bech32Address>> = {
        let lookup = HashMap::new();
        Mutex::new(lookup)
    };
}


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .at("/links/", post(create_link))
        .at("/locks/", post(create_lock))
        .at("/locks/", delete(delete_lock))
        .at("/items/", post(create_item))
        .at("/items/", delete(delete_item));

    let url = format!("127.0.0.1:{}", API_PORT);
    Server::new(TcpListener::bind(url)).run(app).await
}


#[derive(Deserialize)]
struct CreateLinkRequest {
    player: String,
    wallet: String,
}

#[derive(Serialize)]
struct CreateLinkResponse {
    player: String,
    wallet: String,
}

#[handler]
fn create_link(req: Json<CreateLinkRequest>) -> Result<Json<CreateLinkResponse>> {
    let wallet = match Bech32Address::from_str(&req.wallet) {
        Ok(wallet) => wallet,
        Err(err) => return Err(BadRequest(err)),
    };

    let mut lookup = WALLET_LOOKUP.lock().unwrap();
    match lookup.get(&req.player) {
        Some(_) => {
            return Err(Conflict(Error::new(
                ErrorKind::AlreadyExists,
                "player already linked to wallet",
            )))
        }
        None => (),
    }

    lookup.insert(req.player.clone(), wallet);

    let res = CreateLinkResponse {
        player: req.player.clone(),
        wallet: req.wallet.clone(),
    };

    Ok(Json(res))
}

#[derive(Deserialize)]
struct CreateLockRequest {
    player: String,
}

#[derive(Serialize)]
struct CreateLockResponse {
    player: String,
    locked: bool,
    logs: Vec<String>,
}


#[handler]
async fn create_lock(req: Json<CreateLockRequest>) -> Result<Json<CreateLockResponse>> {
    let wallet = get_wallet(&req.player).await?;
    let fuelscape = get_contract().await?;

    let lock = fuelscape.methods().lock(wallet.into());
    let result = match lock.call().await {
        Ok(result) => result,
        Err(err) => return Err(InternalServerError(err)),
    };

    let res = CreateLockResponse {
        player: req.player.clone(),
        locked: true,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

     Ok(Json(res))
}


#[derive(Deserialize)]
struct DeleteLockRequest {
    player: String,
}

#[derive(Serialize)]
struct DeleteLockResponse {
    player: String,
    locked: bool,
    logs: Vec<String>,
}


#[handler]
async fn delete_lock(req: Json<DeleteLockRequest>) -> Result<Json<DeleteLockResponse>> {
    let wallet = get_wallet(&req.player).await?;
    let fuelscape = get_contract().await?;

    let lock = fuelscape.methods().unlock(wallet.into());
    let result = match lock.call().await {
        Ok(result) => result,
        Err(err) => return Err(InternalServerError(err)),
    };

    let res = DeleteLockResponse {
        player: req.player.clone(),
        locked: false,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

     Ok(Json(res))
}

#[derive(Deserialize)]
struct CreateItemRequest {
    player: String,
    item: u64,
    amount: u64,
}

#[derive(Serialize)]
struct CreateItemResponse {
    player: String,
    item: u64,
    amount: u64,
    logs: Vec<String>,
}

#[handler]
async fn create_item(req: Json<CreateItemRequest>) -> Result<Json<CreateItemResponse>> {
    let wallet = get_wallet(&req.player).await?;
    let fuelscape = get_contract().await?;
    
    let give = fuelscape.methods().give(wallet.into(), req.item, req.amount);
    let result = match give.call().await {
            Ok(result) => result,
            Err(err) => return Err(InternalServerError(err)),
    };

    let res = CreateItemResponse {
        player: req.player.clone(),
        item: req.item,
        amount: result.value,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

    Ok(Json(res))
}

#[derive(Deserialize)]
struct DeleteItemRequest {
    player: String,
    item: u64,
    amount: u64,
}

#[derive(Serialize)]
struct DeleteItemResponse {
    player: String,
    item: u64,
    amount: u64,
    logs: Vec<String>,
}

#[handler]
async fn delete_item(req: Json<DeleteItemRequest>) -> Result<Json<DeleteItemResponse>> {
    let wallet = get_wallet(&req.player).await?;
    let fuelscape = get_contract().await?;
    
    let take = fuelscape.methods().take(wallet.into(), req.item, req.amount);
    let result = match take.call().await {
            Ok(result) => result,
            Err(err) => return Err(InternalServerError(err)),
    };

    let res = DeleteItemResponse {
        player: req.player.clone(),
        item: req.item,
        amount: result.value,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

    Ok(Json(res))
}

async fn get_wallet(player: &String) -> Result<Bech32Address> {
    let lookup = WALLET_LOOKUP.lock().unwrap();
    match lookup.get(player) {
        Some(wallet) => Ok(wallet.clone()),
        None => {
            return Err(NotFound(Error::new(
                ErrorKind::NotFound,
                "player not linked to wallet",
            )))
        }
    }
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