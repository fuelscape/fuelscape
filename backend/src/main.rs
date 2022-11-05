use std::collections::HashMap;

use serde::Deserialize;

use poem::error::Error;
use poem::error::Conflict;
use poem::get;
use poem::handler;
use poem::listener::TcpListener;
use poem::post;
use poem::Result;
use poem::Route;
use poem::Server;
use poem::web::Json;

use fuel_gql_client::fuel_tx::Address;

#[derive(Deserialize,Serialize)]
struct Link {
    player: String,
    wallet: Address,
}

struct LinkController {
    lookup: HashMap<String, Address>,
}

impl LinkController {
    #[handler]
    fn create(&self, req: Json<Link>) -> Result<Json<Link>> {
        match self.lookup.get(req.player) {
            Some(wallet) => Err(Conflict(String::from("player already linked to wallet"))),
            _ => (),
        }
    
        self.lookup.insert(req.player, req.wallet);
    
        Ok(Json(req))
    }

    #[handler]
    fn list(&self) -> Result<Json<Vec<Link>>> {
    
    }
}

#[tokio::main]
async fn main() -> Result<(),Error> {

    let lookup = HashMap::new();

    let links = LinkController{
        lookup: lookup,
    };

    let app = Route::new()
        .at("/links/", post(links.create))
        .at("/links/", get(links.list));

    Server::new(TcpListener::bind("127.0.0.1:8080"))
        .run(app)
        .await
}