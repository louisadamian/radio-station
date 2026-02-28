use tokio;
use clap::{self, Parser};
use crate::write_aprs;
use std::error::Error;
#[derive(Parser, Debug)]
#[command(name="APRS parser", version=clap::crate_version!(), about="parses APRS data from KISS over TCP connection and adds data to json file", long_about = None)]
struct Args{
    #[clap(short='a', long="address", default_value="127.0.0.1:8001")]
    url: String,
    #[clap(short='p', default_value="../static/stations.json")]
    path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    println!("connecting to {}", args.url);
    write_aprs(args.path, args.url).await
}

