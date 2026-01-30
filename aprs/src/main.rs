use aprs_parser::{self, AprsPacket, Timestamp};
use ax25::frame::Ax25Frame;
use chrono;
use chrono::{DateTime, Datelike, SecondsFormat, TimeZone, Utc};
use kiss_tnc::Tnc;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::result::Result;
use tokio::time::sleep;
use std::time::{Duration, Instant};
use tokio;
use clap::{self, Parser};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
fn serialize_time<S>(value: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(value.to_rfc3339_opts(SecondsFormat::Secs, true).as_str())
}
#[derive(Clone, Debug, Serialize)]
struct AprsData {
    callsign: String,
    packet: String,
    lat: f64,
    lon: f64,
    #[serde(serialize_with = "serialize_time")]
    time: DateTime<Utc>,
}

async fn cleanup(
    mut dict: HashMap<String, AprsData>,
    evict_time: chrono::TimeDelta,
) -> Result<HashMap<String, AprsData>, Box<dyn Error>> {
    println!("Cleaning up...");
    for (k, v) in dict.clone() {
        if k == "KD4AAA-1" {
            println!("Cleaning up {:?}", k);
        }
        if Utc::now() - v.time > evict_time {
            println!("removed {:}", k);
            dict.remove(&k);
        }
    }
    Ok(dict)
}
fn parse_timestamp(timestamp: Timestamp) -> DateTime<Utc> {
    let now = Utc::now();
    match timestamp {
        Timestamp::DDHHMM(d, h, m) => Utc
            .with_ymd_and_hms(now.year(), now.month(), d as u32, h as u32, m as u32, 0)
            .unwrap(),
        Timestamp::HHMMSS(h, m, s) => Utc
            .with_ymd_and_hms(
                now.year(),
                now.month(),
                now.day(),
                h as u32,
                m as u32,
                s as u32,
            )
            .unwrap(),
        Timestamp::Unsupported(_) => Utc::now(),
    }
}

// fn parse_symbol(table: char, code:char) -> Result<String, Box<dyn Error>> {
//     match table { '/' => {
//         match code {
//             '\'' => {}
//             _ => {}
//         }
//     }
//         _ => {}
//     }
// }
#[derive(Parser, Debug)]
struct Args{
    #[clap(short='u', long, default_value="127.0.0.1:8343")]
    url: String,
    #[clap(short='p', long, default_value="../static/stations.json")]
    path: String,
}
async fn try_connect(url: String) ->Tnc<OwnedReadHalf, OwnedWriteHalf>{
    let mut tnc;
    loop {
        tnc = Tnc::connect_tcp(&url).await;
        match tnc {
            Ok(_) => {
                return tnc.unwrap();
            }
            Err(_) => {}
        }
        sleep(Duration::from_millis(250)).await;
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut tnc = try_connect(args.url).await;
    let mut stations: HashMap<String, AprsData> = HashMap::new();
    let mut last_cleanup = Instant::now();
    let mut last_write = Instant::now();
    let write_interval = Duration::from_secs(1);
    let cleanup_interval = Duration::from_secs(15);
    let evict_time = chrono::TimeDelta::seconds(15);

    let json_file = "../static/stations.json";

    loop {
        match tnc.read_frame().await {
            Ok((_port, data)) => match AprsPacket::decode_ax25(data.as_slice()) {
                Ok(packet) => match packet.clone().data {
                    aprs_parser::AprsData::Position(position) => {
                        let lat = position.latitude.value();
                        let lon = position.longitude.value();
                        let symbol = position.symbol_code;
                        println!("symbol = {:#?}", symbol);
                        println!("symbol_table = {:#?}", position.symbol_table);
                        let name = packet.from.to_string();
                        let packet = Ax25Frame::from_bytes(data.as_slice())?.to_string();
                        if stations.contains_key(&name) {
                            let s = stations.get_mut(&name).unwrap();
                            s.lat = lat;
                            s.lon = lon;
                            s.packet = packet;
                            s.time = match position.timestamp {
                                Some(ts) => parse_timestamp(ts),
                                None => Utc::now(),
                            }
                        } else {
                            stations.insert(
                                name.clone(),
                                AprsData {
                                    callsign: name,
                                    packet,
                                    lat,
                                    lon,
                                    time: match position.timestamp {
                                        Some(ts) => parse_timestamp(ts),
                                        None => Utc::now(),
                                    },
                                },
                            );
                        }
                        // println!("{:?}", Ax25Frame::from_bytes(data.as_slice())?.to_string());
                        let list = stations.values().cloned().collect::<Vec<AprsData>>();
                        println!("{:?}", list);
                        let json = serde_json::to_string(&list)?;
                        let mut f = File::create(json_file)?;
                        f.write_all(json.as_bytes())?;
                        f.sync_all()?;
                    }
                    _ => {}
                },
                _ => break,
            },
            Err(_) => {
                break;
            }
        }
        if last_cleanup.elapsed() > cleanup_interval {
            stations = cleanup(stations, evict_time).await?;
            last_cleanup = Instant::now();
        }
        if last_write.elapsed() > write_interval {
            let json = serde_json::to_string_pretty(
                &stations.values().cloned().collect::<Vec<AprsData>>(),
            )?;
            let mut f = File::create(json_file)?;
            f.write_all(json.as_bytes())?;
            f.sync_all()?;
            last_write = Instant::now();
        }
    }
    println!("caught error. exiting");
    Ok(())
}
