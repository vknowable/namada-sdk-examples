use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::types::token::Transfer;
use namada_tx::data::TxType;
use namada_tx::proto::Tx;
use namada_tx::{Section, Tx as TxStruct};
use prost::Message;

fn main() {
  // you can get this string from a regular json rpc query, eg: http://localhost:26657/block
  // this example is a token transfer
  const TX_STRING: &str = "CvoGHgAAAHNoaWVsZGVkLWV4cGVkaXRpb24uYjQwZDhlOTA1NQAjAAAAMjAyNC0wMi0wMlQxNzo0MjozNi42ODYwNzQ1NjkrMDA6MDCJkSfXIuV4orfAGVYxqI1obMiEeRrMxFkudJYGcu7znoirva8c0EwMmKfW+zpqMByLnH2SxMkMQAA7JFgTiJIiL+Tf1B3E72SH/NYsRT9NbIjL1c6DcOYpXz3G704E7v8BZAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGAJiNVDBl3c2yO3DHJOBPvFcVerdyAJkBMo/osfB19B5AzOb2daYf1tfS5xZ2QVvZyHT7kH/LAQAAAAAAAAAgTgAAAAAAAAAFAAAAAY5N6mqNAQAAARQAAABUcmFuc2ZlciBmcm9tIGZhdWNldAACvk3qao0BAAAA7sN/NxKXZuipvYkIxO6fY3cj3vC1OzN90ETmduMDuCoBEAAAAHR4X3RyYW5zZmVyLndhc20Avk3qao0BAABiAAAAAavPS82xpUcwpuOIoGd9zkDa0eSCAGdqu1EaZYZgBydtIiskFeJauzi4AJiNVDBl3c2yO3DHJOBPvFcVerdyAMqaOwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGAAADAQAAAMVX/5Br7rzeoFfAwJzbxpIkQQIdfVXYm85PX9K3tFuXAAGrz0vNsaVHMKbjiKBnfc5A2tHkggEAAAAAAHM6YVnbU0gQKoHvAv1NknY9iPUCCiOG3apyvOpbmWnbfdCBe4cAykHyAJ5RoqHxgPISVFZxhJFpvCDtqnPNUQgDBQAAAG+sknVcUS+7OvjMCwld0YXKFagxNBDWRG3CP6GlwO24L+Tf1B3E72SH/NYsRT9NbIjL1c6DcOYpXz3G704E7v+JkSfXIuV4orfAGVYxqI1obMiEeRrMxFkudJYGcu7znoirva8c0EwMmKfW+zpqMByLnH2SxMkMQAA7JFgTiJIidGITD17miEfLZanv9H4+IPJvxg9hD/e299c2NbghZQkBAQAAAACZATKP6LHwdfQeQMzm9nWmH9bX0ucWdkFb2ch0+5B/ywEAAAAAAPEmTr6GDCTmF/1pqq+wxHEzPuFGRwStifQAyalottUjwUFScqgC//oJib92FF5ey2YMgHPPvHAxmCab8qGN4Qw=";

  // decode from base64 string to bytes
  let tx_bytes: Vec<u8> = base64::decode(TX_STRING).unwrap();

  // use prost to further decompose the raw bytes into the protobuf type
  let tx_from_bytes: Tx = Tx::decode(&tx_bytes[..]).unwrap();

  // finally use borsh to deserialize the data bytes to get the rust Tx struct
  let decoded_tx: TxStruct = TxStruct::try_from_slice(&tx_from_bytes.data).unwrap();

  // Tx type can be 'Wrapper', 'Decrypted', 'Protocol', and 'Raw'; each have different structure
  match decoded_tx.header().tx_type {
    TxType::Wrapper(_) => {
      println!("TxType: Wrapper");
        // check the sections[1] field ("code field") to get the code tag, if you want to know the tx type. This example is 'tx_transfer.wasm'
        // sections[2] field has the serialized tx data bytes
        // (there is probably a better/safer way to access this data)
        if let Section::Code(code) = decoded_tx.sections[1].clone() {
          println!("tx type: {:?}", code.tag.unwrap());
        }
        let transfer_data = decoded_tx.sections[2].data().unwrap().data;

        // finally we can deserialize the data section of the transfer to get the human readable components
        let transfer: Transfer = Transfer::try_from_slice(&transfer_data[..]).unwrap();
        println!("{:?}", transfer);

        // using the example tx string at the start of main(), we should end up here
        // expected output:
        // Transfer { source: Implicit: tnam1qz4u7j7dkxj5wv9xuwy2qemaeeqd450ysgl7pq0r, target: Established: tnam1q9nk4w63rfjcvcq8yakjy2eyzh394wechqyv7flr,
        //  token: Established: tnam1qxvg64psvhwumv3mwrrjfcz0h3t3274hwggyzcee, amount: DenominatedAmount { amount: Amount { raw: 1000000000 }, 
        //  denom: Denomination(6) }, key: None, shielded: None }
    }
    TxType::Decrypted(_) => {
      // TODO: What is "Decrypted" tx type and how to use it?
      println!("TxType: Decrypted");
      println!("contents: {:?}", decoded_tx);
    }
    TxType::Protocol(tx) => {
      // Protocol tx includes stuff like validator set updates, eth bridge events etc
      println!("TxType: Protocol ({:?})", tx.tx);
    }
    TxType::Raw => {
      // TODO: What is "Raw" tx type and how to use it?
      println!("TxType: Raw");
    }
  }
}
