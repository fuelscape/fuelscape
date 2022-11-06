#[macro_use]
extern crate lazy_static;

use std::boxed::Box;
use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::str::FromStr;
use std::sync::Mutex;

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

use poem::error::Conflict;
use poem::error::BadGateway;
use poem::error::BadRequest;
use poem::error::NotFound;
use poem::error::InternalServerError;

const API_PORT: &str = "8080";
const NODE_URL: &str = "node-beta-1.fuel.network";
const WALLET_MNEMONIC: &str = "wet person force drum vicious milk afraid target treat verify faculty dilemma forget across congress visa hospital skull twenty sick ship tent limit survey";
const CONTRACT_ID: &str = "0xeadd5a23608fbde7c787590d1c6925a958fdb627e0f03c39edb06d65aa140d25";

abigen!(FuelScape, "../contract/out/debug/fuelscape-abi.json");

lazy_static! {
    static ref WALLET_LOOKUP: Mutex<HashMap<String, String>> = {
        let lookup = HashMap::new();
        Mutex::new(lookup)
    };
    static ref PLAYER_LOOKUP: Mutex<HashMap<String, String>> = {
        let lookup = HashMap::new();
        Mutex::new(lookup)
    };
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let cors = Cors::new().allow_methods([Method::GET, Method::POST, Method::DELETE]);

    let app = Route::new()
        .at("/", get(version))
        .at("/links/", post(create_link))
        .at("/links/:wallet", get(retrieve_link))
        .at("/locks/", post(create_lock).delete(delete_lock))
        .at("/items/", post(create_item).delete(delete_item))
        .at("/items/:player", get(list_items))
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
struct CreateLinkRequest {
    player: String,
    wallet: String,
}

#[derive(Serialize)]
struct CreateLinkResponse {
    player: String,
    wallet: String,
    linked: bool,
}

#[handler]
async fn create_link(req: Json<CreateLinkRequest>) -> Result<Json<CreateLinkResponse>> {
    let _address = match Bech32Address::from_str(&req.wallet) {
        Ok(address) => address,
        Err(err) => return Err(BadRequest(err)),
    };

    let mut wallets = WALLET_LOOKUP.lock().unwrap();
    match wallets.get(&req.player) {
        Some(_) => {
            return Err(Conflict(Error::new(
                ErrorKind::AlreadyExists,
                "player already linked to wallet",
            )))
        }
        None => (),
    }

    let mut players = PLAYER_LOOKUP.lock().unwrap();
    match players.get(&req.wallet) {
        Some(_) => {
            return Err(Conflict(Error::new(
                ErrorKind::AlreadyExists,
                "wallet already linked to player",
            )))
        }
        None => (),
    }

    wallets.insert(req.player.clone(), req.wallet.clone());
    players.insert(req.wallet.clone(), req.player.clone());

    let res = CreateLinkResponse {
        player: req.player.clone(),
        wallet: req.wallet.clone(),
        linked: true,
    };

    Ok(Json(res))
}

#[derive(Serialize)]
struct RetrieveLinkResponse {
    player: String,
    wallet: String,
    linked: bool,
}

#[handler]
async fn retrieve_link(Path(wallet): Path<String>) -> Result<Json<RetrieveLinkResponse>> {
    let player = get_player(&wallet).await?;

    let res = RetrieveLinkResponse{
        player: player.clone(),
        wallet: wallet.clone(),
        linked: true,
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
    wallet: String,
    locked: bool,
    logs: Vec<String>,
}

#[handler]
async fn create_lock(req: Json<CreateLockRequest>) -> Result<Json<CreateLockResponse>> {
    let wallet = get_wallet(&req.player).await?;
    let fuelscape = get_contract().await?;

    let address = match Bech32Address::from_str(&wallet) {
        Ok(address) => address,
        Err(err) => return Err(InternalServerError(err)),
    };

    let lock = fuelscape
        .methods()
        .lock(address.into())
        .tx_params(TxParameters::new(Some(1), Some(1000000), None));
    let result = match block_on(lock.call()) {
        Ok(result) => result,
        Err(err) => return Err(InternalServerError(err)),
    };

    let res = CreateLockResponse {
        player: req.player.clone(),
        wallet: wallet.clone(),
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
    wallet: String,
    locked: bool,
    logs: Vec<String>,
}

#[handler]
async fn delete_lock(req: Json<DeleteLockRequest>) -> Result<Json<DeleteLockResponse>> {
    let wallet = get_wallet(&req.player).await?;
    let fuelscape = get_contract().await?;

    let address = match Bech32Address::from_str(&wallet) {
        Ok(address) => address,
        Err(err) => return Err(InternalServerError(err)),
    };

    let lock = fuelscape
        .methods()
        .unlock(address.into())
        .tx_params(TxParameters::new(Some(1), Some(1000000), None));
    let result = match block_on(lock.call()) {
        Ok(result) => result,
        Err(err) => return Err(InternalServerError(err)),
    };

    let res = DeleteLockResponse {
        player: req.player.clone(),
        wallet: wallet.clone(),
        locked: false,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

    Ok(Json(res))
}

#[derive(Deserialize)]
struct CreateItemRequest {
    player: String,
    item: u16,
    amount: u32,
}

#[derive(Serialize)]
struct CreateItemResponse {
    player: String,
    wallet: String,
    item: u16,
    balance: u32,
    logs: Vec<String>,
}

#[handler]
async fn create_item(req: Json<CreateItemRequest>) -> Result<Json<CreateItemResponse>> {
    let wallet = get_wallet(&req.player).await?;
    let fuelscape = get_contract().await?;

    let address = match Bech32Address::from_str(&wallet) {
        Ok(address) => address,
        Err(err) => return Err(InternalServerError(err)),
    };

    let give = fuelscape
        .methods()
        .give(address.into(), req.item, req.amount)
        .tx_params(TxParameters::new(Some(1), Some(1000000), None));
    let result = match block_on(give.call()) {
        Ok(result) => result,
        Err(err) => return Err(InternalServerError(err)),
    };

    let res = CreateItemResponse {
        player: req.player.clone(),
        wallet: wallet.clone(),
        item: req.item,
        balance: result.value,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

    Ok(Json(res))
}

#[derive(Deserialize)]
struct DeleteItemRequest {
    player: String,
    item: u16,
    amount: u32,
}

#[derive(Serialize)]
struct DeleteItemResponse {
    player: String,
    wallet: String,
    item: u16,
    balance: u32,
    logs: Vec<String>,
}

#[handler]
async fn delete_item(req: Json<DeleteItemRequest>) -> Result<Json<DeleteItemResponse>> {
    let wallet = get_wallet(&req.player).await?;
    let fuelscape = get_contract().await?;

    let address = match Bech32Address::from_str(&wallet) {
        Ok(address) => address,
        Err(err) => return Err(InternalServerError(err)),
    };

    let take = fuelscape
        .methods()
        .take(address.into(), req.item, req.amount)
        .tx_params(TxParameters::new(Some(1), Some(1000000), None));
    let result = match block_on(take.call()) {
        Ok(result) => result,
        Err(err) => return Err(InternalServerError(err)),
    };

    let res = DeleteItemResponse {
        player: req.player.clone(),
        wallet: wallet.clone(),
        item: req.item,
        balance: result.value,
        logs: fuelscape.fetch_logs(&result.receipts),
    };

    Ok(Json(res))
}

#[derive(Serialize)]
struct ListItemsResponse {
    player: String,
    wallet: String,
    balances: Vec<Balance>,
}

#[derive(Serialize)]
struct Balance {
    item: u16,
    balance: u32,
}

#[handler]
async fn list_items(Path(player): Path<String>) -> Result<Json<ListItemsResponse>> {
    let wallet = get_wallet(&player).await?;
    let fuelscape = get_contract().await?;

    let address = match Bech32Address::from_str(&wallet) {
        Ok(address) => address,
        Err(err) => return Err(InternalServerError(err)),
    };

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
        .map(|entry| Balance{
            item: entry.item,
            balance: entry.balance,
        })
        .collect();

    let res = ListItemsResponse {
        player: player.clone(),
        wallet: wallet.clone(),
        balances,
    };

    Ok(Json(res))
}

async fn get_wallet(player: &String) -> Result<String> {
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

async fn get_player(address: &String) -> Result<String> {
    let lookup = PLAYER_LOOKUP.lock().unwrap();
    match lookup.get(address) {
        Some(player) => Ok(player.clone()),
        None => {
            return Err(NotFound(Error::new(
                ErrorKind::NotFound,
                "wallet not linked to player",
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
