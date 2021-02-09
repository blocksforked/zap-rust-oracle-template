
use ethabi::Token;
use web3::contract::{Contract, Options};
use web3::futures::{future, StreamExt};
use web3::contract::tokens::{Detokenize, Tokenize};
use web3::types::{Address, BlockNumber, Log, U256,FilterBuilder};
use web3::api::{Accounts};
use web3::types::{FilterBuilder, Log, H256,H160};

pub async fn make_callback(address:H160,id:H256,value:Token) -> Result<(), Box<dyn std::error::Error>> {
    let client = web3::Web3::new(web3::transports::WebSocket::new(dotenv!("INFURA")).await?);
    println!("{}", "starting");
    let c_address: Address = "5E92eD766BeeEc15da1023c80Ba7157eF276A3e6".parse().unwrap();
    println!("{}", "starting");

    let nonce = client
        .eth()
        .transaction_count(
            "2465fD0fdD229CC3F14F098865fB1d45973B09EE".parse().unwrap(),
            Some(BlockNumber::Latest),
        )
        .await?;

    let Dcontract = Contract::from_json(
        client.eth(),
        c_address,
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
   let secret_key = SecretKey::from_slice(&hex::decode( &hex::decode(private_key).unwrap()).unwrap()).unwrap();
  
   let args:Vec<Token>=[Token::Uint(id),Token::String(value)];

    println!("{}", "making call");
    
    let tx_hash = Dcontract
        .signed_call_with_confirmations(
            "Callback",
            args,
            _options,
            confirmations,
            &secret_key,
        )
        .await?;

    println!("{}", "done");
    //let _receipt = contract.pending_transaction(tx_hash).await.unwrap();
    println!("{:?}", tx_hash);
    Ok(())
}
