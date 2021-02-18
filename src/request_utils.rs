


use serde_json::Value;
use substring::Substring;

async fn make_get(
    url: &String,
    
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let echo_json: serde_json::Value = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .json()
        .await?;

    println!("{:#?}", &echo_json);

    Ok(echo_json)
}
async fn make_post(
    url: &String,
    body: Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let echo_json: serde_json::Value = reqwest::Client::new()
        .post(url)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    println!("{:#?}", &echo_json);

    Ok(echo_json)
}
pub fn buildPath(args:Vec<String>)->String{
    let size=args.len();
    let mut path=String::new();
   
    let mut concat=|i:usize,x:String|{
        if(i==0){
            path.push_str("/");
        }
        path.push_str(&x);
        println!("{:?}",i);
        if(i<size-1){
            path.push_str("/");
        }
    };
    args.into_iter().enumerate().for_each(|(i, x)| concat(i,x) ); 
    path
        // Insert a string at the end o
}
pub fn processEndpointParams(raw:String)->Vec<String>{
    let process_hex=|x:&&str|->String{
        let temp=String::from_utf8(hex::decode(x).unwrap()).unwrap();
        let s:Vec<&str>=temp.split('\u{0}').collect();
        String::from(s[0])
    };
    let length=&raw.len()-1;
    let temp=raw.substring(1,length).to_string();
    let s:Vec<&str>=temp.split(',').collect();
    let decode_params=s.iter().map(|x| process_hex(x)).collect();
    decode_params
}

pub async fn fetchAndProcessQuery(raw_params:String,query:String)->String{
    let raw_path=processEndpointParams(raw_params);
    let pointer_path=buildPath(raw_path);
    println!("{:?}","QUERY");
    println!("{:?}",&query);
    let json_result=make_get(&query).await;
    json_result.unwrap().pointer(&pointer_path).unwrap().to_string()

}