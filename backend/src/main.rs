#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::sync::Mutex;

use serde::Deserialize;
use serde::Serialize;

use poem::error::Conflict;
use poem::handler;
use poem::listener::TcpListener;
use poem::post;
use poem::Result;
use poem::Route;
use poem::Server;
use poem::web::Json;

use fuel_gql_client::fuel_tx::Address;

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

#[tokio::main]
async fn main() -> Result<(),std::io::Error> {

    let app = Route::new()
        .at("/links/", post(create_link));

    Server::new(TcpListener::bind("127.0.0.1:8080"))
        .run(app)
        .await
}