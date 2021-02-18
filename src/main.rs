mod config_loader;
mod event_decoder;
mod request_utils;
mod contracts;
extern crate secp256k1;
use config_loader::{event_config, load_config};
use dotenv::dotenv;
use event_decoder::{decode_log,event_value};
use contracts::{make_callback};
use request_utils::{buildPath,processEndpointParams,fetchAndProcessQuery};
use serde_json::Value;
use std::collections::HashMap;
use substring::Substring;

use tiny_keccak::Keccak;

use web3::futures::{future, StreamExt};
use web3::types::{FilterBuilder, Log, H256,H160,U256,U64};
use std::str::FromStr;
extern crate rustc_hex;
use run_script::ScriptOptions;
use rustc_hex::ToHex;
use std::env;
extern crate eth_checksum;
use std::sync::Arc;
//use async_std::{net::TcpStream, prelude::*};
//use io_arc::IoArc;

#[macro_use]
extern crate dotenv_codegen;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let args: Vec<String> = env::args().collect();

    println!("{:?}", &args);
    let private_key: &[u8] = dotenv!("PRIVATE_KEY").as_bytes();
    let provider = &args[1];
    let provider_arc= Arc::new(provider.clone());
    let endpoint=dotenv!("ENDPOINT");
    //let socket_port = &args[3];
    println!("{:?}", &provider);
    let mut event_mapping = HashMap::new();

    let web3 = web3::Web3::new(web3::transports::WebSocket::new(provider).await?);

    let (events, signatures, addresses) = load_config(&args[2]);

    events.into_iter().enumerate().for_each(|(_key, item)| {
        println!("{:?}", &item.event_hash);
        println!("{:?}", &item.address);
        println!("{:?}", eth_checksum::checksum( &item.address));
        let address_string=H160::from_str(&item.address).unwrap();
        println!("{:?}",eth_checksum::checksum(&format!("{:x}", address_string)).as_str());
        let hash_key = event_key(&item.event_hash,  eth_checksum::checksum(&format!("{:x}", address_string)).as_str() );
        println!("the hash key is {:?}", hash_key);
        event_mapping.insert(hash_key, item);
    });
    //println!("{:?}",&addresses);
    // println!("{:?}",&signatures);
    let filter = FilterBuilder::default()
        .address(addresses)
        .topics(Some(signatures), None, None, None)
        .build();
    println!("{:?}", &filter);
    let sub = web3.eth_subscribe().subscribe_logs(filter).await?;
    //let stream = async_std::net::TcpStream::connect(socket_port).await;

    //let arc_stream = IoArc::new(stream.unwrap());
   
    sub.for_each(|log| async {
        let _log = log;
        let response_data:Vec<event_value> =
        process_log(_log.unwrap(), &event_mapping).await;

        if(response_data[3].value==endpoint){
            let provider_copy  = Arc::clone(&provider_arc);        
           
            tokio::spawn(async move {
                println!("{:?}","starting callback");
                oracleCallback(response_data,&provider_copy).await;
            });

        }else{
            println!("{}","wrong endpoint called")
        }

        future::ready(()).await
    })
    .await;

    Ok(())
}

async fn oracleCallback(data:Vec<event_value>,provider:&String){
    let params=String::from(&data[5].value);
    let subscriber=H160::from_str(&data[2].value).unwrap();
    let id=U256::from_str(&data[0].value).unwrap();
    let query=data[3].value.clone();
    let query_result=fetchAndProcessQuery(params,query).await;
    println!("{:?}",query_result);
    make_callback(subscriber,id,query_result,provider).await; 
}

fn event_key(sig: &String, address: &str) -> String {
    let mut result = [0u8; 32];
    let mut n = String::new();
    n.push_str(address);
    n.push_str(sig);
    let data = n.replace(" ", "").into_bytes();
    let mut sponge = Keccak::new_keccak256();
    sponge.update(&data);
    sponge.finalize(&mut result);
    format!("{:x}", H256::from_slice(&result))
}

async fn process_log<'a>(
    log: Log,
    map: &'a HashMap<String, event_config>,
) -> (Vec<event_value>) {
    let event = log;
    let event_copy = event.clone();
    println!("got log: {:?}", &event);
    let topics: Vec<String> = event_copy
        .topics
        .into_iter()
        .map(|topic| format!("{:x}", topic))
        .collect();
    println!("{:?}", &topics[0]);
    println!("{:?}", eth_checksum::checksum(&format!("{:x}", &event.address)).as_str());

    let key = event_key(&topics[0],  eth_checksum::checksum(&format!("{:x}", &event.address)).as_str() );
    println!("the key is {:?}", key);

    let config = map.get(&key).unwrap();
    let decoded_log = decode_log(
        &config.abi_path,
        &config.name,
        topics,
        &event.data.0.to_hex::<String>(),
    )
    .unwrap();

   decoded_log
}

#[test]
fn Incoming_log_decode_path_parsing() {
    use serde_json::json;
    let abi = "./eventsABI/Incoming.abi";
    let name = "Incoming";
    let topic = [
        String::from("69741cc3ec0270f258feb6b53b42ef1e7d2251a3c8eea4f6ba1f72bd4b7beba7"),
        String::from("dd01e0d1e313c493bd8dcb841088d6d6bcbca3b0c3cfe6d0c76df566f0b2577d"),
        String::from("000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266"),
        String::from("000000000000000000000000959922be3caee4b8cd9a407cc3ac1c251c2007b1"),
    ];
    let data = "00000000000000000000000000000000000000000000000000000000000000800da72197e898ebe1814471a76048ed137a089f595c1e7038f2b70e98645e765200000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000057175657279000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000270617468310000000000000000000000000000000000000000000000000000007061746832000000000000000000000000000000000000000000000000000000";
    //let expected = [event_value::new(String::from("id"),String::from("dd01e0d1e313c493bd8dcb841088d6d6bcbca3b0c3cfe6d0c76df566f0b2577d")),event_value::new_from_str("provider","f39fd6e51aad88f6f4ce6ab8827279cfffb92266"),event_value::new_from_str("query","query"),event_value::new_from_str("endpoint","0da72197e898ebe1814471a76048ed137a089f595c1e7038f2b70e98645e7652" ),event_value::new_from_str("endpointParams","[273c58c66704975459b45d17d8d262d7b6de59f5e57291ea5af890e2cf11a2b8,08455b9a244e3f6e53a4d620c3d1261bb351f203ccdf11324cfed2696b108913]")];

    let decoded = decode_log(abi, name, topic.to_vec(), data).unwrap();
    println!("{:?}", &decoded[5].value);
    println!("{:?}", String::from(&decoded[5].value).len());
   
    let path_params=processEndpointParams(String::from(&decoded[5].value));
    println!("{:?}",path_params);
    let path=buildPath(path_params);
    let data = json!({
        "path1": {
            "path2": "value to get"
        }
    });
    println!("{:?}",&path);
    let p=data.pointer(&path).unwrap();
    println!("{:?}",p);
    
    assert_eq!(p.as_str(), Some("value to get"));

    //cd buildPath(args:Vec<String>)->
    //println!("{:?}", post_body.to_string());
   // println!("{:?}", post_string);
}
#[test]

fn Send_Decoded_Event_websocket() {
    use std::io::prelude::*;
    use std::net::TcpStream as syncTcpStream;
    let abi = "./eventsABI/Incoming.abi";
    let name = "Incoming";
    let topic = [
        String::from("69741cc3ec0270f258feb6b53b42ef1e7d2251a3c8eea4f6ba1f72bd4b7beba7"),
        String::from("dd01e0d1e313c493bd8dcb841088d6d6bcbca3b0c3cfe6d0c76df566f0b2577d"),
        String::from("000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266"),
        String::from("0000000000000000000000009a9f2ccfde556a7e9ff0848998aa4a0cfd8863ae"),
    ];
    let data = "00000000000000000000000000000000000000000000000000000000000000800da72197e898ebe1814471a76048ed137a089f595c1e7038f2b70e98645e765200000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000571756572790000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002273c58c66704975459b45d17d8d262d7b6de59f5e57291ea5af890e2cf11a2b808455b9a244e3f6e53a4d620c3d1261bb351f203ccdf11324cfed2696b108913";

    let decoded = decode_log(abi, name, topic.to_vec(), data).unwrap();
    //let mut stream = syncTcpStream::connect("127.0.0.1:3007").unwrap();
   // let post_vec = serde_json::to_vec(&decoded).unwrap();
    //println!("{:?}",&c);
    //stream.write(&post_vec);

}

#[tokio::test]
async fn testcallback(){
    use tokio_test::block_on;
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
      }
    
    let subscriber=H160::from_str("f39fd6e51aad88f6f4ce6ab8827279cfffb92266").unwrap();
    let id=U256::from_str("32453840958").unwrap();
    let query=String::from("this is a result");
    let provider=String::from("ws://127.0.0.1:8545");
    let result=make_callback(subscriber,id,query,&provider).await;

    assert_eq!(U64::from(1),result.unwrap().status.unwrap());

}
#[tokio::test]
async fn testOracleCallback(){
    use serde_json::json;
    let abi = "./eventsABI/Incoming.abi";
    let name = "Incoming";
    let topic = [
        String::from("69741cc3ec0270f258feb6b53b42ef1e7d2251a3c8eea4f6ba1f72bd4b7beba7"),
        String::from("dd01e0d1e313c493bd8dcb841088d6d6bcbca3b0c3cfe6d0c76df566f0b2577d"),
        String::from("000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266"),
        String::from("000000000000000000000000959922be3caee4b8cd9a407cc3ac1c251c2007b1"),
    ];
    let data = "00000000000000000000000000000000000000000000000000000000000000800da72197e898ebe1814471a76048ed137a089f595c1e7038f2b70e98645e765200000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004868747470733a2f2f6170692e636f696e6765636b6f2e636f6d2f6170692f76332f73696d706c652f70726963653f6964733d616176652676735f63757272656e636965733d757364000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000261617665000000000000000000000000000000000000000000000000000000007573640000000000000000000000000000000000000000000000000000000000";
    //let expected = [event_value::new(String::from("id"),String::from("dd01e0d1e313c493bd8dcb841088d6d6bcbca3b0c3cfe6d0c76df566f0b2577d")),event_value::new_from_str("provider","f39fd6e51aad88f6f4ce6ab8827279cfffb92266"),event_value::new_from_str("query","query"),event_value::new_from_str("endpoint","0da72197e898ebe1814471a76048ed137a089f595c1e7038f2b70e98645e7652" ),event_value::new_from_str("endpointParams","[273c58c66704975459b45d17d8d262d7b6de59f5e57291ea5af890e2cf11a2b8,08455b9a244e3f6e53a4d620c3d1261bb351f203ccdf11324cfed2696b108913]")];

    let decoded = decode_log(abi, name, topic.to_vec(), data).unwrap();
    println!("{:?}",decoded);
    let provider=String::from("ws://127.0.0.1:8545");
    oracleCallback(decoded ,&provider).await;
    assert!(true);

}

