use axum:: {
    routing::{get, post},
    Router,
    Json,
    response::IntoResponse
};
use serde::{Serialize, Deserialize};

use solana_sdk::{
  signature::{Keypair, Signer},
};

use bs58;




#[derive(Serialize)]
struct SuccessResponse<T> {
    success: bool,
    data: T,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}




// 1st 1st api and required strucutures, couldn't create required file!
#[derive(Serialize)]
struct WalletResponse {
    pubkey : String,
    secret_key : String
}
pub async fn create_wallet() -> impl IntoResponse {
    let keypair = Keypair::new();
    let public_key = keypair.pubkey().to_string();
    let private_key_in_bytes = keypair.to_bytes();
    let final_private_key = bs58::encode(private_key_in_bytes).into_string();

    let res = WalletResponse {
        pubkey : public_key,
        secret_key : final_private_key
    };
    Json(SuccessResponse {
        success: true,
        data: res,
    })
}


// now wedo 2nd api stuff




#[tokio::main]
async fn main() {
    let app = Router::new().route("/check-health", get(check_health))
        .route("/keypair", post(create_wallet));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn check_health() -> String {
    "The server is working at 3000".to_string()
}