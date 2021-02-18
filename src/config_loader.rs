use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::io::Error;
use std::path::Path;
use web3::types::{H160, H256,U256};

pub struct event_config {
    pub name: String,
    pub event_hash: String,
    pub address: String,
    pub abi_path: String,
    pub response_type: String,
    pub response_data: String,
}
pub struct oracleConfig{
    pub title: String,
    pub public_key:String,
    pub NODE_URL:String,
    pub STATUS_URL:String,
    pub EndpointSchema:endpointParams,

}
pub struct endpointParams{
    pub name:String,
    pub curve:Vec<u128>,
    pub broker:String,
    pub md:String,
    pub queries:Vec<queryConfig>
}
pub struct queryConfig{
    pub query:String,
    pub params:Vec<String>,
    pub dynamic:bool,
    pub responseType:String,
}
impl  oracleConfig{
    fn new(
        _title: String,
        _public_key: String,
        _NODE_URL:String,
        _STATUS_URL: String,
       _EndpointSchema:endpointParams,
    )->oracleConfig{
        oracleConfig{
        title:_title,
        public_key:_public_key,
        NODE_URL:_NODE_URL,
        STATUS_URL:_STATUS_URL,
        EndpointSchema:_EndpointSchema
        }
    }
}
impl queryConfig{
    fn new(
        _query: String,
        _params: Vec<String>,
        _dynamic:bool,
        _responseType: String,
       
    ) -> queryConfig{
        queryConfig{
            query: _query,
            params:_params,
            dynamic: _dynamic,
            responseType: _responseType,            
        }
    }
}
impl endpointParams{
    fn new(
        _name: String,
        _curve: Vec<u128>,
        _broker:String,
        _md: String,
        _queries:Vec<queryConfig>
       
    ) ->  endpointParams{
        endpointParams {
            name: _name,
            curve:_curve,
            broker: _broker,
            md: _md,   
            queries:_queries         
        }
    }

}
impl event_config {
    fn new(
        _name: String,
        _event_hash: String,
        _address: String,
        _abi_path: String,
        _response_type: String,
        _response_data: String,
    ) -> event_config {
        event_config {
            name: _name,
            event_hash: _event_hash,
            address: _address,
            abi_path: _abi_path,
            response_type: _response_type,
            response_data: _response_data,
        }
    }
}
fn read_config_from_file(_path: &str) -> Result<Value, Box<Error>> {
    // Open the file in read-only mode with buffer.
    let path = Path::new(_path);
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as a SERDE Value type.
    let config = serde_json::from_reader(reader).unwrap();

    // Return the Value`.
    Ok(config)
}
//&'static String
pub fn convert_value_to_event_config(json: Value) -> (Vec<event_config>, Vec<H256>, Vec<H160>) {
    let config_array = &json["events"];
    let mut address = Vec::new();
    let mut event_sigs = Vec::new();
    let mut event_configs = Vec::new();

    config_array
        .as_array()
        .unwrap()
        .into_iter()
        .for_each(|event| {
            println!("{:?}", &event);
            event_configs.push(event_config::new(
                String::from(event["name"].as_str().unwrap()),
                String::from(event["event_hash"].as_str().unwrap()),
                String::from(event["address"].as_str().unwrap()),
                String::from(event["abi_path"].as_str().unwrap()),
                String::from(event["response_type"].as_str().unwrap()),
                String::from(event["response_data"].as_str().unwrap()),
            ));
            println!("{:?}", event["address"].to_string());

            let temp: H160 = event["address"].as_str().unwrap().parse().unwrap();

            let temp2: H256 = event["event_hash"].as_str().unwrap().parse().unwrap();

            address.push(H160::from(temp));
            event_sigs.push(H256::from(temp2));
        });
    (event_configs, event_sigs, address)
}
pub fn load_config(path: &String) -> (Vec<event_config>, Vec<H256>, Vec<H160>) {
    let raw_values = read_config_from_file(path.as_str()).unwrap();
    convert_value_to_event_config(raw_values)
}
