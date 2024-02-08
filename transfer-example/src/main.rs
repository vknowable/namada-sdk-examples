use dotenvy::dotenv;

use std::str::FromStr;
use tendermint_rpc::{HttpClient, Url};
use namada_sdk::{
	args::{InputAmount, TxBuilder}, io::NullIo, masp::fs::FsShieldedUtils, rpc, signing::default_sign, tendermint::abci::Code, types::{
		address::Address, chain::ChainId, key::{common::SecretKey, RefTo}, masp::{TransferSource, TransferTarget}, token::{Amount, NATIVE_SCALE},
	}, wallet::fs::FsWalletUtils, Namada, NamadaImpl
};
use namada_tx::data::ResultCode;


#[tokio::main]
async fn main() {

    // parse the .env file and use to populate values for rpc client, source's private key, target, token, amount, and chain-id
	let config = AppConfig::parse();
	let url = Url::from_str(&config.rpc).expect("invalid RPC address");
	let http_client = HttpClient::new(url).unwrap();

	let token = Address::decode(config.token.clone());
	let token = if let Ok(address) = token {
        address
    } else {
        panic!("Invalid token address");
    };

	let sk = SecretKey::from_str(&config.private_key)
		.expect("Should be able to decode secret key.");
	let source = Address::from(&sk.ref_to());

	let target = Address::decode(config.target.clone());
	let target = if let Ok(address) = target {
		address
	} else {
		panic!("Invalid target address")
	};

    // create a chain context to store our keys and construct/sign our transfer
	let wallet = FsWalletUtils::new("wallet".into());
	let shielded_ctx = FsShieldedUtils::new("masp".into());

	let sdk = NamadaImpl::new(http_client, wallet, shielded_ctx, NullIo)
		.await
		.expect("unable to initialize Namada context")
		.chain_id(ChainId::from_str(&config.chain_id).unwrap());

    // add our source private key into our wallet
    sdk.wallet_mut().await.insert_keypair(
        "my_wallet".to_string(),
        true,
        sk,
        None,
        Some(source.clone()),
        None, 
    );

    // give our token amount the right number of decimal places
    // ie: NAM has 6 decimal places, so 100NAM => 100 000 000 tokens on chain
    let amt = rpc::denominate_amount(
        sdk.client(),
        sdk.io(),
        &token,
        Amount::from_u64(config.amount*NATIVE_SCALE)
    )
    .await;

    println!("source: {:?} target: {:?} amount:{} token {}", source, target, amt.to_string(), token.to_string());

    // populate our Tx args -- the things you would normally enter at the cli: source, target, token, amount, memo
	let mut transfer_tx_builder = sdk.new_transfer(	
        TransferSource::Address(source),
        TransferTarget::Address(target.clone()),
        token.clone(),
        InputAmount::Unvalidated(amt),
    );
	transfer_tx_builder.tx.memo = Some("Test transfer".to_string().as_bytes().to_vec());

    // build and sign the tx for broadcast
	let (mut transfer_tx, signing_data, _epoch) = transfer_tx_builder
        .build(&sdk)
        .await
        .expect("unable to build transfer");
    sdk.sign(
            &mut transfer_tx,
            &transfer_tx_builder.tx,
            signing_data,
            default_sign,
            (),
        )
        .await
        .expect("unable to sign reveal pk tx");

    // submit the tx to chain and await the response
    let process_tx_response = sdk.submit(transfer_tx, &transfer_tx_builder.tx).await;

	let (sent, tx_hash) = if let Ok(response) = process_tx_response {
        match response {
            namada_sdk::tx::ProcessTxResponse::Applied(r) => (r.code.eq(&ResultCode::Ok), Some(r.hash)),
            namada_sdk::tx::ProcessTxResponse::Broadcast(r) => {
                (r.code.eq(&Code::Ok), Some(r.hash.to_string()))
            }
            _ => (false, None),
        }
    } else {
        (false, None)
    };

    // if successful, "sent: true" "tx: [tx hash]"
	println!("sent: {}", sent);
	println!("tx: {}", tx_hash.unwrap());	
}

// a convenience struct to hold our .env file values
struct AppConfig {
    rpc: String,
    private_key: String,
    target: String,
    chain_id: String,
    amount: u64,
    token: String,
}

impl AppConfig {
    fn parse() -> Self {
        // Load environment variables from .env file
        dotenv().ok();

        // Read configuration values from environment variables
        let rpc = dotenvy::var("RPC_ADDRESS").expect("RPC address not found in environment");
        let private_key = dotenvy::var("PRIVATE_KEY").expect("Private key not found in environment");
        let target = dotenvy::var("TARGET_ADDRESS").expect("Target address not found in environment");
        let chain_id = dotenvy::var("CHAIN_ID").expect("Chain Id not found in environment");
        let token = dotenvy::var("TOKEN").expect("Token not found in environment");
        let amount = u64::from_str(
            dotenvy::var("AMOUNT").expect("Amount not found in environment")
            .as_str())
            .expect("Invalid token amount found in environment");

        // Create and return an instance of AppConfig
        AppConfig {
            rpc,
            private_key,
            target,
            chain_id,
            amount,
            token,
        }
    }
}
