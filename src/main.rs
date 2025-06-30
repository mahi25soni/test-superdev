use axum:: {
    routing::{get, post},
    Router,
    Json,
    response::IntoResponse
};
use serde::{Serialize, Deserialize};

use solana_sdk::{
    signature::Signature,
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
    secret : String
}
pub async fn create_wallet() -> impl IntoResponse {
    let keypair = Keypair::new();
    let public_key = keypair.pubkey().to_string();
    let private_key_in_bytes = keypair.to_bytes();
    let final_private_key = bs58::encode(private_key_in_bytes).into_string();

    let res = WalletResponse {
        pubkey : public_key,
        secret : final_private_key
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


     let mint = match Pubkey::from_str(&payload.mint) {
        Ok(pk) => pk,
        Err(_) => {
            return Json(ErrorResponse {
                success: false,
                error: "Invalid mint public key".into(),
            }).into_response();
        }
    };

    let mint_authority = match Pubkey::from_str(&payload.mintAuthority) {
        Ok(pk) => pk,
        Err(_) => {
            return Json(ErrorResponse {
                success: false,
                error: "Invalid mint authority public key".into(),
            }).into_response();
        }
    };

    // Validate decimals
    if payload.decimals > 18 {
        return Json(ErrorResponse {
            success: false,
            error: "Decimals must be between 0 and 18".into(),
        }).into_response();
    }

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
    }).into_response()
}



// now 3rd api
#[derive(Deserialize)]
struct MintTokenRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

use spl_token::instruction::mint_to;

pub async fn mint_token(Json(payload): Json<MintTokenRequest>) -> impl IntoResponse {
    // Validate `mint`
    let mint = match Pubkey::from_str(&payload.mint) {
        Ok(pk) => pk,
        Err(_) => {
            return Json(ErrorResponse {
                success: false,
                error: "Invalid mint public key".into(),
            }).into_response();
        }
    };

    // Validate `destination`
    let destination = match Pubkey::from_str(&payload.destination) {
        Ok(pk) => pk,
        Err(_) => {
            return Json(ErrorResponse {
                success: false,
                error: "Invalid destination public key".into(),
            }).into_response();
        }
    };

    // Validate `authority`
    let authority = match Pubkey::from_str(&payload.authority) {
        Ok(pk) => pk,
        Err(_) => {
            return Json(ErrorResponse {
                success: false,
                error: "Invalid authority public key".into(),
            }).into_response();
        }
    };

    // Validate `amount`
    if payload.amount == 0 {
        return Json(ErrorResponse {
            success: false,
            error: "Amount must be greater than 0".into(),
        }).into_response();
    }

    // Create instruction
    let instruction = match mint_to(
        &spl_token::id(), // Token Program ID
        &mint,
        &destination,
        &authority,
        &[], // no multisig
        payload.amount,
    ) {
        Ok(ix) => ix,
        Err(_) => {
            return Json(ErrorResponse {
                success: false,
                error: "Failed to create mint_to instruction".into(),
            }).into_response();
        }
    };

    // Convert accounts
    let accounts: Vec<SingleAccountResponse> = instruction.accounts.iter().map(|acc| {
        SingleAccountResponse {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        }
    }).collect();

    let res = CreateTokenResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: base64::encode(instruction.data),
    };

    Json(SuccessResponse {
        success: true,
        data: res,
    }).into_response()
}



// 4th one
#[derive(Deserialize)]
struct SignMessageRequest {
    message: String,
    secret: String,
}

#[derive(Serialize)]
struct SignMessageResponse {
    signature: String,    
    public_key: String,   
    message: String,
}

async fn sign_message(Json(payload): Json<SignMessageRequest>) -> impl IntoResponse {
    // Decode the base58 secret key
    let secret_bytes = match bs58::decode(&payload.secret).into_vec() {
        Ok(bytes) if bytes.len() == 64 => bytes,
        _ => {
            return Json(ErrorResponse {
                success: false,
                error: "Invalid or malformed secret key".into(),
            }).into_response();
        }
    };

    // Reconstruct Keypair from 64-byte secret
    let keypair = match Keypair::from_bytes(&secret_bytes) {
        Ok(kp) => kp,
        Err(_) => {
            return Json(ErrorResponse {
                success: false,
                error: "Failed to construct keypair from secret".into(),
            }).into_response();
        }
    };

    // Sign the message
    let signature = keypair.sign_message(payload.message.as_bytes());

    // Prepare response
    let res = SignMessageResponse {
        signature: base64::encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: payload.message,
    };

    Json(SuccessResponse {
        success: true,
        data: res,
    }).into_response()
}


// 5thhh
#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: String,
    signature: String, 
    pubkey: String,   
}

#[derive(Serialize)]
struct VerifyMessageResponse {
    valid: bool,
    message: String,
    pubkey: String,
}

pub async fn verify_message(Json(payload): Json<VerifyMessageRequest>) -> impl IntoResponse {
    // Decode the public key
    let pubkey = match bs58::decode(&payload.pubkey).into_vec() {
        Ok(bytes) => match Pubkey::try_from(bytes.as_slice()) {
            Ok(pk) => pk,
            Err(_) => {
                return Json(ErrorResponse {
                    success: false,
                    error: "Invalid public key".into(),
                }).into_response();
            }
        },
        Err(_) => {
            return Json(ErrorResponse {
                success: false,
                error: "Failed to decode public key from base58".into(),
            }).into_response();
        }
    };

    // Decode the base64 signature
    let signature_bytes = match base64::decode(&payload.signature) {
        Ok(sig) => sig,
        Err(_) => {
            return Json(ErrorResponse {
                success: false,
                error: "Failed to decode signature from base64".into(),
            }).into_response();
        }
    };

    let signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(_) => {
            return Json(ErrorResponse {
                success: false,
                error: "Invalid signature format".into(),
            }).into_response();
        }
    };

    // Perform verification
    let valid = signature.verify(pubkey.as_ref(), payload.message.as_bytes());

    let res = VerifyMessageResponse {
        valid,
        message: payload.message,
        pubkey: payload.pubkey,
    };

    Json(SuccessResponse {
        success: true,
        data: res,
    }).into_response()
}
#[tokio::main]
async fn main() {
    let app = Router::new().route("/check-health", get(check_health))
        .route("/keypair", post(create_wallet))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn check_health() -> String {
    "The server is working at 3000".to_string()
}