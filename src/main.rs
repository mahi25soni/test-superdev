use axum:: {
    routing::{get, post},
    Router,
    Json,
    response::IntoResponse
};
use serde::{Serialize, Deserialize};

use solana_sdk::{
  signature::{Keypair, Signer},
      instruction::{AccountMeta, Instruction},
    system_program,
};
use solana_program::pubkey::Pubkey;
use spl_token::instruction::initialize_mint;

use bs58;
use base64;
use std::str::FromStr;




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
#[derive(Deserialize)]
struct CreateTokenRequest {
    mintAuthority: String,
    mint: String,
    decimals: u8,
}

#[derive(Serialize)]
struct SingleAccountResponse {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Serialize)]
struct CreateTokenResponse {
    program_id: String,
    accounts: Vec<SingleAccountResponse>,
    instruction_data: String,
}


pub async fn create_token(Json(payload): Json<CreateTokenRequest>) -> impl IntoResponse {
    let mint = Pubkey::from_str(&payload.mint).unwrap();
    let mint_authority = Pubkey::from_str(&payload.mintAuthority).unwrap();
    let rent_sysvar = solana_sdk::sysvar::rent::id();

    // Create the instruction
    let instruction = initialize_mint(
        &spl_token::id(),          // SPL Token Program ID
        &mint,
        &mint_authority,
        None,                      // freeze authority optional
        payload.decimals,
    ).unwrap();

    // Convert AccountMeta to serializable version
    let accounts: Vec<SingleAccountResponse> = instruction.accounts.iter().map(|meta| {
        SingleAccountResponse {
            pubkey: meta.pubkey.to_string(),
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        }
    }).collect();

    // Convert instruction data to base64
    let instruction_data = base64::encode(instruction.data);

    let res = CreateTokenResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data,
    };

    Json(SuccessResponse {
        success: true,
        data: res,
    })
}




#[tokio::main]
async fn main() {
    let app = Router::new().route("/check-health", get(check_health))
        .route("/keypair", post(create_wallet))
        .route("/token/create", post(create_token));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn check_health() -> String {
    "The server is working at 3000".to_string()
}