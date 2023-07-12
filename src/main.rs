use actix_web::{error, post, web, App, Error, HttpResponse, HttpServer};
use dotenv::dotenv;
use ethers::prelude::Address;
use serde::Deserialize;
use std::env;

use crate::sign_data::{sign, Presale};
mod sign_data;

#[derive(Deserialize)]
pub struct PresaleRequest {
    pub presale: Presale,
    pub owner: Address,
}

/// Handles the signing operation for a presale event.
///
/// This function receives the details of a presale and the owner's address, and attempts to sign it.
/// If the signing operation is successful, the signature is returned in the response body.
/// In case of a failure, an HTTP BadRequest response is returned with an "overflow" error message.
///
/// # Arguments
///
/// * `data` - A JSON payload encapsulated in `web::Json<PresaleRequest>`. This includes the details of a presale and the owner's address.
///
/// # Returns
///
/// * On success, returns `Ok(HttpResponse::Ok().body(signature))` where `signature` is the signed presale details.
/// * On error, returns `Err(error::ErrorBadRequest("overflow"))`.
///
/// # Errors
///
/// Returns an error if:
///
/// * The `sign` function fails to sign the presale.
#[post("/sign")]
async fn signer(data: web::Json<PresaleRequest>) -> Result<HttpResponse, Error> {
    let body = data.into_inner();
    let presale: Presale = body.presale;
    let owner: Address = body.owner;
    match sign(presale, owner).await {
        Ok(signature) => Ok(HttpResponse::Ok().body(signature)),
        Err(_) => Err(error::ErrorBadRequest("overflow")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let mut port = "8000".to_string();
    match env::var("PORT") {
        Ok(res) => {
            port = res;
        }
        Err(_) => {}
    }
    let server_address = format!("127.0.0.1:{port}");
    let result = HttpServer::new(|| App::new().service(signer)).bind(server_address);
    match result {
        Ok(server) => {
            println!("HTTP server successfully started on {}", port);
            server.run().await
        }
        Err(e) => {
            println!("Failed to start the HTTP server: {}", e);
            Err(e)
        }
    }
}
