use std::io::Write;

use clap::Parser;

use log::{self, info};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    about,
    version,
    after_help = "
    You can specify the chain id by alias or by id, supported aliases are:
    eth: Ethereum
    op: Optimism
    bsc: Binance Smart Chain
    poly: Polygon
    base: Base
    arb: Arbitrum
    lin: Linea
    linea: Linea
    era: ZkSync Era
    zksync: ZkSync Era
    "
)]
struct Args {
    /// Chain id or alias, for more info see the help
    chain: String,
    /// Address of the contract to clone
    address: String,

    /// Path to clone the contract to
    path: String,
}

fn chain_to_id(key: &str) -> Option<i32> {
    match key {
        "eth" => Some(1),
        "op" => Some(10),
        "bsc" => Some(51),
        "poly" => Some(137),
        "base" => Some(8453),
        "arb" => Some(42161),
        "lin" => Some(59144),
        "linea" => Some(59144),
        "era" => Some(324),
        "zksync" => Some(324),
        _ => None,
    }
}

fn build_url(chainid: i32, baseurl: &str, address: &str) -> String {
    let mut url = baseurl.to_string();
    url.push_str("?chainid=");
    url.push_str(&chainid.to_string());
    url.push_str("&module=contract&action=getsourcecode&address=");
    url.push_str(address);
    url
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_target(false)
        .format_timestamp(None)
        .init();
    let args = Args::parse();

    //check if supplied dir exists, if it does abort, if it doesnt create it
    if std::path::Path::new(&args.path).exists() {
        panic!("Path already exists");
    }
    std::fs::create_dir(&args.path).unwrap();
    //parsing the chain id parameter or converting the chain name to id
    let chainid: i32;
    if args.chain.parse::<i32>().is_ok() {
        chainid = args.chain.parse().unwrap();
    } else {
        let chainidopt = chain_to_id(&args.chain);
        if chainidopt.is_none() {
            panic!("Invalid chain id");
        }
        chainid = chainidopt.unwrap();
    }
    info!("Chain id: {}", chainid);
    info!(
        "Cloning contract at address {} to path {}",
        args.address, args.path
    );
    //initialize the forge project
    let mut child = std::process::Command::new("forge")
        .arg("init")
        .arg(args.path.clone())
        .arg("--no-commit")
        .spawn()
        .unwrap();
    child.wait().unwrap();

    let url = build_url(chainid, "https://api.etherscan.io/v2/api", &args.address);
    info!("URL: {}", url);
    //whole bunch of weird json parsing to get the contract source code
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await.unwrap();
    let body = res.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let result = json["result"].as_array().unwrap();
    let mut contract = result[0]["SourceCode"].as_str().unwrap().to_string();
    contract = contract.replace("{{", "{");
    contract = contract.replace("}}", "}");
    let contractobj: serde_json::Value = serde_json::from_str(&contract).unwrap();
    let sources = contractobj["sources"].as_object().unwrap();

    //deleting all the counter files
    for entry in WalkDir::new(&args.path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.to_str().unwrap().contains("Counter") {
            std::fs::remove_file(path).unwrap();
        }
    }
    //iterating over our contract sources and creating the files
    for (key, value) in sources {
        let mut srcpath = format!("{}/{}", args.path, "src");

        srcpath.push_str("/");

        let mut dirs = key.split("/").collect::<Vec<&str>>();
        dirs.pop();
        for dir in dirs {
            srcpath.push_str(dir);
            srcpath.push_str("/");
            std::fs::create_dir_all(&srcpath).unwrap();
        }

        let name = key.split("/").last().unwrap();
        srcpath.push_str(name);

        let mut file = std::fs::File::create(srcpath).unwrap();
        file.write_all(value["content"].as_str().unwrap().as_bytes())
            .unwrap();
    }
}
