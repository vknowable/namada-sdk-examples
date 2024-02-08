// sdk is divided into modules for reading (ie querying chain) and transaction (constructing the various types of tx; transfer, ibc, governance etc)
// you can also choose between blocking and non-blocking (async/await)
use namada_light_sdk::reading::asynchronous::{query_block, pos};
// the sdk also re-exports the core namada types for usage
use namada_light_sdk::namada_sdk::types::storage::BlockHeight;
use tokio;

#[tokio::main]
async fn main() {
  // our rpc/full node
  const URL: &str = "localhost:26657";

  // query some stuff from the chain and print the results
  let height = match query_block(URL).await.expect("Could not get last block") {
    Some(block) => block.height,
    None => BlockHeight::from(0),
  };
  let current_epoch = pos::query_epoch(URL).await.expect("Could not query epoch");
  let validator_set = pos::get_all_validators(URL, current_epoch).await.expect("Could not get validator set");
  
  println!("Hello world from Namada!");
  println!("Height: {}", height);
  println!("Epoch: {}", current_epoch);
  println!("Validator set:");
  for validator in validator_set.iter() {
    let stake = pos::get_validator_stake(URL, current_epoch, validator).await.unwrap();
    println!("{} / {} vp", validator, stake);
  }
}