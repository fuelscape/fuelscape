#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::str::FromStr;
use std::sync::Mutex;

use serde::Deserialize;
use serde::Serialize;

use poem::error::BadGateway;
use poem::error::BadRequest;
use poem::error::Conflict;
use poem::error::InternalServerError;
use poem::error::NotFound;
use poem::handler;
use poem::listener::TcpListener;
use poem::post;
use poem::web::Json;
use poem::Result;
use poem::Route;
use poem::Server;

use fuel_crypto::SecretKey;
use fuel_gql_client::fuel_tx::ContractId;
use fuels_signers::provider::Provider;
use fuels_signers::wallet::DEFAULT_DERIVATION_PATH_PREFIX;
use fuels_signers::WalletUnlocked;
use fuels_types::bech32::Bech32Address;

const API_PORT: &str = "8080";
const NODE_URL: &str = "node-beta-1.fuel.network";
const WALLET_MNEMONIC: &str = "wet person force drum vicious milk afraid target treat verify faculty dilemma forget across congress visa hospital skull twenty sick ship tent limit survey";
const CONTRACT_ID: &str = "0xb6aa962b6538fa6951def08282136df7d2dec885102d1c9a6ec8f7e0701ba2b3";

use fuels::prelude::*;
abigen!(FuelScape,"../contract/out/debug/fuelscape-abi.json");

lazy_static! {
    static ref WALLET_LOOKUP: Mutex<HashMap<String, Bech32Address>> = {
        let lookup = HashMap::new();
        Mutex::new(lookup)
    };
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
    let wallet = {
        let lookup = WALLET_LOOKUP.lock().unwrap();
        match lookup.get(&req.player) {
            Some(wallet) => wallet.clone(),
            None => {
                return Err(NotFound(Error::new(
                    ErrorKind::NotFound,
                    "player not linked to wallet",
                )))
            }
        }
    };

    let provider = match Provider::connect(NODE_URL).await {
        Ok(provider) => provider,
        Err(err) => return Err(BadGateway(err)),
    };

    let path = format!("{}/{}'/0/0", DEFAULT_DERIVATION_PATH_PREFIX, 0);

    let raw: Vec<u8> = WALLET_MNEMONIC.bytes().collect();
    let phrase = match String::from_utf8(raw) {
        Ok(phrase) => phrase,
        Err(err) => return Err(InternalServerError(err)),
    };

    let secret = match SecretKey::new_from_mnemonic_phrase_with_path(&phrase, &path) {
        Ok(secret) => secret,
        Err(err) => return Err(InternalServerError(err)),
    };

    let unlocked = WalletUnlocked::new_from_private_key(secret, Some(provider));

    let address = match ContractId::from_str(CONTRACT_ID) {
        Ok(address) => address,
        Err(msg) => return Err(InternalServerError(Error::new(
            ErrorKind::InvalidInput,
            msg,
        ))),
    };
    
    let recipient = match Bech32Address::from_str(&req.player) {
        Ok(recipient) => recipient,
        Err(err) => return Err(BadRequest(err)),
    };

    let fuelscape = FuelScape::new(address.into(), unlocked);


    let result = match fuelscape.methods()
        .give(recipient.into(), req.item, req.amount)
        .call()
        .await {
            Ok(result) => result,
            Err(err) => return Err(InternalServerError(err)),
    };

    let logs = fuelscape.fetch_logs(&result.receipts);


    let res = CreateItemResponse {
        player: req.player.clone(),
        item: req.item,
        amount: req.amount,
        logs: logs,
    };

    Ok(Json(res))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .at("/links/", post(create_link))
        .at("/items/", post(create_item));

    let url = format!("127.0.0.1:{}", API_PORT);
    Server::new(TcpListener::bind(url)).run(app).await
}
