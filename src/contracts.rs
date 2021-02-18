
use secp256k1::key::SecretKey;
use ethabi::Token;
use web3::contract::{Contract, Options};
use web3::futures::{future, StreamExt};
use web3::contract::tokens::{Detokenize, Tokenize};
use web3::signing::SecretKeyRef;
use web3::types::{Address, BlockNumber,TransactionReceipt, Log, U256,H256,H160,FilterBuilder};
use web3::api::Accounts;


extern crate dotenv;
use dotenv::dotenv;


pub async fn make_callback(address:H160,id:U256,value:String,port:&String) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
    let transport = web3::transports::WebSocket::new(&port).await?;
    let client = web3::Web3::new(web3::transports::WebSocket::new(&port).await?);
    println!("{}", "starting");
    //let c_address: Address = "5E92eD766BeeEc15da1023c80Ba7157eF276A3e6".parse().unwrap();
    println!("{}", "starting");

    let nonce = client
        .eth()
        .transaction_count(
            "f39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse().unwrap(),
            Some(BlockNumber::Latest),
        )
        .await?;

    let Dcontract = Contract::from_json(
        client.eth(),
        address,
        include_bytes!("../contracts/OffchainClient.abi"),
    )
    .unwrap();
    

    println!("{}", nonce);

    
    let confirmations: usize = 1;
    
    let mut _options = Options::default();
    _options.gas = Some( 0xff11.into() );
    _options.nonce = Some( U256::from(nonce) );
    _options.gas_price = Some( 0x09184e72au64.into() );
    
    //let accounts = Accounts::new(client.eth().transport().clone()); 
     
   // let accounts = client.eth().accounts().await?;
   // accounts.sign("helloword",private_key);
   let private_key: &[u8] = dotenv!("PRIVATE_KEY").as_bytes();
   let secret_key:SecretKey = SecretKey::from_slice(&hex::decode(private_key).unwrap()).unwrap();
   //println!("{:?}",secret_key.address());
   //let KEY=SecretKeyRef::new(&secret_key);
   //let signed_event=client.accounts().sign(String::from("hello").into_bytes(),KEY  );
  
   //let args=[Token::Uint(id),Token::String(value)];

    println!("{}", "making call");
    
    let tx_hash = Dcontract
        .signed_call_with_confirmations(
            "respond1",
            (id,value),
            _options,
            confirmations,
            &secret_key,
        )
        .await?;

    println!("{}", "done");
    //let _receipt = contract.pending_transaction(tx_hash).await.unwrap();
    println!("{:?}", &tx_hash);
    
    Ok(tx_hash)
}
