
    use radio_station_aprs::write_aprs;
    use std::process::Command;
    use std::fs;
    use tokio;
    use serde::Deserialize;
    use serde_json::{Result, Value};

    #[derive(Debug,Deserialize)]
    struct Aprs{
        callsign:String,
        packet: String,
        lat: f64,
        lon: f64,
        time: String,
    }
    #[tokio::test]
    async fn test_aprs_parser() {
        let path = "stations_test.json".to_string();
        let c = Command::new("python3").arg("gen_aprs.py").spawn();
        write_aprs(path.clone(), "127.0.0.1:8001".to_string()).await.unwrap();
        c.unwrap().wait().unwrap();
        let content = fs::read_to_string(path).unwrap();
        let json:Value = serde_json::from_str(&content).unwrap();
        for stations in json.as_array().unwrap() {
            match stations["callsign"].as_str().unwrap() {
                "KD4AAA-3"=>{
                    // assert_eq!(json[0]["callsign"], "KD4AAA-1".to_string());
                    assert_eq!(stations["symbol"], "/#".to_string());
                    let lat = stations["lat"].as_f64().unwrap();
                    let lon = stations["lon"].as_f64().unwrap();
                    assert!(lat>42.3629&&lat<42.3631, "latitude of {} incorrect",lat);
                    assert!(lon< -71.1256666&&lon> -71.12566667, "latitude of {} incorrect",lon);
                }
                _ =>{}
            }
        }

    }
