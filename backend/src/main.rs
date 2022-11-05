#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::sync::Mutex;

use serde::Deserialize;
use serde::Serialize;

use poem::error::BadGateway;
use poem::error::Conflict;
use poem::error::InternalServerError;
use poem::error::NotFound;
use poem::handler;
use poem::listener::TcpListener;
use poem::post;
use poem::Result;
use poem::Route;
use poem::Server;
use poem::web::Json;

use fuel_crypto::SecretKey;
use fuels_signers::provider::Provider;
use fuels_signers::WalletUnlocked;
use fuels_signers::wallet::DEFAULT_DERIVATION_PATH_PREFIX;
use fuel_gql_client::fuel_tx::Address;

const NODE_URL: &str = "node-beta-1.fuel.network";

const WALLET_MNEMONIC: &str = "wet person force drum vicious milk afraid target treat verify faculty dilemma forget across congress visa hospital skull twenty sick ship tent limit survey";

lazy_static! {
    static ref WALLET_LOOKUP: Mutex<HashMap<String, Address>> = {
        let lookup = HashMap::new();
        Mutex::new(lookup)
    };
}

#[derive(Deserialize)]
struct CreateLinkRequest {
    player: String,
    wallet: Address,
}

#[derive(Serialize)]
struct CreateLinkResponse {
    player: String,
    wallet: Address,
}

#[handler]
fn create_link(req: Json<CreateLinkRequest>) -> Result<Json<CreateLinkResponse>> {

    let mut lookup = WALLET_LOOKUP.lock().unwrap();
    match lookup.get(&req.player) {
        Some(_) => return Err(Conflict(Error::new(
                ErrorKind::AlreadyExists,
                "player already linked to wallet",
            ))),
        None => (),
    }

    lookup.insert(req.player.clone(), req.wallet);

    let res = CreateLinkResponse{
        player: req.player.clone(),
        wallet: req.wallet,
    };

    Ok(Json(res))
}

#[derive(Deserialize)]
struct CreateKillRequest {
    player: String,
    mob: u64,
}

#[derive(Serialize)]
struct CreateKillResponse {
    player: String,
    mob: u64,
}

#[handler]
async fn create_kill(req: Json<CreateKillRequest>) -> Result<Json<CreateKillResponse>> {

    let wallet = {
        let lookup = WALLET_LOOKUP.lock().unwrap();
        match lookup.get(&req.player) {
            Some(wallet) => wallet.clone(),
            None => return Err(NotFound(Error::new(
                ErrorKind::NotFound,
                "player not linked to wallet",
            ))),
        }
    };

    let provider = match Provider::connect(NODE_URL).await {
        Ok(provider) => provider,
        Err(err) => return Err(BadGateway(err)),
    };

    let path =  format!("{}/{}'/0/0", DEFAULT_DERIVATION_PATH_PREFIX, 0);

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

    let res = CreateKillResponse{
        player: req.player.clone(),
        mob: req.mob,
    };

    Ok(Json(res))
}

#[tokio::main]
async fn main() -> Result<(),std::io::Error> {

    let app = Route::new()
        .at("/links/", post(create_link));

    Server::new(TcpListener::bind("127.0.0.1:8080"))
        .run(app)
        .await
}