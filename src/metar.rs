use chrono::{DateTime, Datelike, SecondsFormat, Timelike, Utc};
#[cfg(feature = "metar-bin")]
use clap::Parser;
use futures_util::StreamExt;
use log;
use metar::{
    CloudLayer, CloudType, Metar, Pressure, VertVisibility, Visibility, WindDirection, WindSpeed,
};
use reqwest;
use serde::Serialize;
use std::cmp::PartialEq;
use std::fs::{self, DirEntry, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::env;
use toml;

fn degrees_c_to_f(temp: f64) -> f64 {
    temp * (9.0 / 5.0) + 32.0
}
fn kts_to_kph(speed: f64) -> f64 {
    speed * 463.0 / 250.0
}
fn kph_to_kts(speed: f64) -> f64 {
    speed * 250.0 / 463.0
}
fn mps_to_kts(speed: f64) -> f64 {
    speed * 900.0 / 463.0
}
fn mps_to_kph(speed: f64) -> f64 {
    speed * 3.6
}
fn ft_to_m(length: f64) -> f64 {
    length * 381.0 / 1250.0
}
fn m_to_mi(length: f64) -> f64 {
    length * 125.0 / 201168.0
}
fn inhg_to_hpa(press: f64) -> f64 {
    press * 33.86388640341
}
fn hpa_to_inhg(press: f64) -> f64 {
    press / 33.86388640341
}

fn parse_visibility(vis: Visibility, units: DisplayUnits) -> Option<f64> {
    if units != DisplayUnits::Metric {
        match vis {
            Visibility::CAVOK => None,
            Visibility::Metres(m) => Option::from(m_to_mi(m as f64)),
            Visibility::StatuteMiles(mi) => Option::from(mi as f64),
        }
    } else {
        match vis {
            Visibility::CAVOK => None,
            Visibility::Metres(m) => Option::from(m_to_mi(m as f64)),
            Visibility::StatuteMiles(mi) => Option::from(f64::from(mi)),
        }
    }
}
fn parse_speed_kts(wind_speed: WindSpeed) -> f64 {
    match wind_speed {
        WindSpeed::Calm => 0.0,
        WindSpeed::Knot(k) => f64::from(k),
        WindSpeed::MetresPerSecond(mps) => mps_to_kts(f64::from(mps)),
        WindSpeed::KilometresPerHour(kph) => kph_to_kts(f64::from(kph)),
    }
}
fn parse_speed_kph(wind_speed: WindSpeed) -> f64 {
    match wind_speed {
        WindSpeed::Calm => 0.0,
        WindSpeed::Knot(kts) => kts_to_kph(f64::from(kts)),
        WindSpeed::MetresPerSecond(mps) => mps_to_kph(f64::from(mps)),
        WindSpeed::KilometresPerHour(kph) => f64::from(kph),
    }
}
fn parse_pressure_hpa(press: Pressure) -> f64 {
    match press {
        Pressure::Hectopascals(hpa) => hpa as f64,
        Pressure::InchesOfMercury(inhg) => inhg_to_hpa(f64::from(inhg)).round(),
    }
}
fn parse_pressure_inhg(press: Pressure) -> f64 {
    match press {
        Pressure::InchesOfMercury(inhg) => inhg as f64,
        Pressure::Hectopascals(hpa) => hpa_to_inhg(f64::from(hpa)),
    }
}
fn parse_wind_dir(dir: WindDirection) -> Option<f64> {
    match dir {
        WindDirection::Heading(h) => Option::from(f64::from(h)),
        WindDirection::Variable => None,
        WindDirection::Above => None,
    }
}
fn parse_wind_status(dir: WindDirection) -> Option<String> {
    match dir {
        WindDirection::Heading(_) => None,
        WindDirection::Variable => Option::from("variable".to_string()),
        WindDirection::Above => Option::from("above".to_string()),
    }
}
fn parse_vert_visibility(vert_visibility: Option<VertVisibility>) -> Option<u32> {
    match vert_visibility {
        None => None,
        Some(vis) => match vis {
            VertVisibility::Distance(vis) => Option::from(vis),
            VertVisibility::ReducedByUnknownAmount => None,
        },
    }
}

#[derive(Debug, Serialize, Clone)]
struct CloudData {
    cloud_type: String,
    density: String,
    height: u32,
}

fn parse_clouds(cloud_layers: Vec<CloudLayer>, units: DisplayUnits) -> Vec<CloudData> {
    let mut cloud_data = Vec::<CloudData>::new();
    for cloud in cloud_layers {
        let s: String;
        let cloud_type: CloudType;
        let cloud_height;
        match cloud.clone() {
            CloudLayer::Few(cl_type, height) => {
                s = "few ".to_string();
                cloud_type = cl_type;
                cloud_height = height;
            }
            CloudLayer::Scattered(cl_type, height) => {
                s = "scattered ".to_string();
                cloud_type = cl_type;
                cloud_height = height;
            }
            CloudLayer::Broken(cl_type, height) => {
                s = "broken ".to_string();
                cloud_type = cl_type;
                cloud_height = height;
            }
            CloudLayer::Overcast(cl_type, height) => {
                s = "overcast ".to_string();
                cloud_type = cl_type;
                cloud_height = height;
            }
            CloudLayer::Unknown(cl_type, height) => {
                s = "".to_string();
                cloud_type = cl_type;
                cloud_height = height;
            }
        }
        let height;
        if units == DisplayUnits::Metric {
            height = ft_to_m(f64::from(cloud_height.unwrap())) as u32;
        } else {
            height = cloud_height.unwrap()
        }

        cloud_data.push(CloudData {
            cloud_type: format!("{:?}", cloud_type),
            density: s,
            height,
        });
    }
    cloud_data
}

#[derive(Debug, Serialize, Clone)]
struct WeatherReport {
    units: DisplayUnits,
    wind_speed: f64,
    wind_gust: Option<f64>,
    wind_dir: Option<f64>,
    wind_status: Option<String>,
    temp: i32,
    dew_point: i32,
    clouds: Vec<CloudData>,
    pressure: f64,
    visibility: Option<f64>,
    vert_visibility: Option<u32>,
    remark: String,
    station: String,
    time: toml::value::Datetime,
}

#[derive(PartialEq, Eq, Debug, Serialize, Clone, Copy)]
pub enum DisplayUnits {
    Imperial,
    Metric,
    Aviation,
    Nautical,
}
fn get_metars(icao: String, dir_path: PathBuf) -> Vec<String> {
    let icao = icao.as_str();
    let mut metars = Vec::new();
    let dir = fs::read_dir(dir_path).unwrap();
    for filepath in dir {
        let f_name = filepath.unwrap().path();
        if !f_name.extension().unwrap().eq_ignore_ascii_case("txt") {
            continue;
        }
        let text = fs::read_to_string(f_name).unwrap();
        if text.contains(icao) {
            let file_metars = text.split("=").collect::<Vec<&str>>();
            let t = file_metars
                .iter()
                .filter(|cur| cur.contains(icao))
                .collect::<Vec<&&str>>()[0]
                .to_string();
            metars.push(
                t.clone()
                    .trim()
                    .to_string()
                    .replace("\n", "")
                    .replace("\r", ""),
            );
        }
    }
    metars
}
fn parse_metar_brief(cur_metar: Metar, units: DisplayUnits) -> String {
    let mut report = String::with_capacity(50);
    if units == DisplayUnits::Nautical || units == DisplayUnits::Imperial {
        report += format!(
            "{:.0}ºF ",
            // uom::si::thermodynamic_temperature::degree_celsius.conversion()
            degrees_c_to_f(f64::from(*cur_metar.temperature.unwrap()))
        )
        .as_str();
    } else {
        report += format!("{:.0}ºC ", f64::from(*cur_metar.temperature.unwrap())).as_str();
    }
    if cur_metar.weather.len() == 0 {
        if cur_metar.cloud_layers.len() == 0 {
            report += "skies clear "
        } else {
            let mut clouds: i32 = 0;
            for cloud in cur_metar.cloud_layers {
                let c = match cloud {
                    CloudLayer::Few(_, _) => 1,
                    CloudLayer::Scattered(_, _) => 2,
                    CloudLayer::Broken(_, _) => 3,
                    CloudLayer::Overcast(_, _) => 4,
                    CloudLayer::Unknown(_, _) => 0,
                };
                if c > clouds {
                    clouds = c;
                }
            }
            match clouds {
                0 => report += "cloudy ",
                1 => report += "Mostly Clear ",
                2 => report += "Partly Cloudy ",
                3 => report += "Mostly Cloudy ",
                4 => report += "overcast ",

                _ => {}
            }
        }
    } else {
        for weather in cur_metar.weather.iter() {
            report += format!("{:?}", weather.intensity).as_str();
            report += " ";
            for condition in weather.conditions.iter() {
                report += format!("{:?}", condition).as_str();
                report += " ";
            }
        }
    }
    report += "winds ";
    if units == DisplayUnits::Nautical || units == DisplayUnits::Aviation {
        let speed = parse_speed_kts(cur_metar.wind.speed.unwrap().clone());
        if speed == 0.0 {
            report += "calm "
        } else {
            report += speed.to_string().as_str();
            report += " kts";
        }

        match cur_metar.wind.gusting {
            Some(c) => {
                report += " gusting ";
                report += parse_speed_kts(c).to_string().as_str();
                report += " kts";
            }
            _ => {}
        }
    } else {
        let speed = parse_speed_kph(cur_metar.wind.speed.unwrap().clone());
        if speed == 0.0 {
            report += "calm "
        } else {
            report += speed.to_string().as_str();
            report += " kph";
        }
        match cur_metar.wind.gusting {
            Some(c) => {
                report += " gusting ";
                report += parse_speed_kph(c).to_string().as_str();
            }
            _ => {}
        }
    }
    report
}

fn parse_time(metar: Metar) -> DateTime<Utc> {
    Utc::now()
        .with_day(metar.time.date as u32)
        .unwrap()
        .with_hour(metar.time.hour as u32)
        .unwrap()
        .with_minute(metar.time.minute as u32)
        .unwrap()
        .with_second(0)
        .unwrap()
}
fn write_report(metar: Metar, units: DisplayUnits) -> WeatherReport {
    let t = parse_time(metar.clone());
    let time =
        toml::value::Datetime::from_str(t.to_rfc3339_opts(SecondsFormat::Secs, true).as_str())
            .unwrap();

    let rep;
    match units {
        DisplayUnits::Metric => {
            rep = WeatherReport {
                units,
                wind_speed: parse_speed_kph(metar.wind.speed.unwrap().clone()),
                wind_gust: match metar.wind.gusting {
                    None => None,
                    Some(speed) => Some(parse_speed_kph(speed)),
                },
                wind_dir: parse_wind_dir(metar.wind.dir.unwrap().clone()),
                wind_status: parse_wind_status(metar.wind.dir.unwrap().clone()),
                temp: *metar.temperature.unwrap(),
                dew_point: *metar.temperature.unwrap(),
                clouds: parse_clouds(metar.cloud_layers, units),
                pressure: parse_pressure_hpa(metar.pressure.unwrap().clone()),
                visibility: parse_visibility(metar.visibility.unwrap().clone(), units),
                vert_visibility: match parse_vert_visibility(metar.vert_visibility) {
                    Some(vis) => Some(ft_to_m(vis as f64) as u32),
                    None => None,
                },
                remark: metar.remarks.unwrap().trim().replace("  ", " ").to_string(),
                station: metar.station,
                time,
            }
        }
        DisplayUnits::Aviation => {
            rep = WeatherReport {
                units,
                wind_speed: parse_speed_kts(metar.wind.speed.unwrap().clone()),
                wind_gust: match metar.wind.gusting {
                    None => None,
                    Some(speed) => Some(parse_speed_kts(speed.clone())),
                },
                wind_dir: parse_wind_dir(metar.wind.dir.unwrap().clone()),
                wind_status: parse_wind_status(metar.wind.dir.unwrap().clone()),
                temp: *metar.temperature.unwrap(),
                dew_point: *metar.temperature.unwrap(),
                clouds: parse_clouds(metar.cloud_layers, units),
                pressure: parse_pressure_inhg(metar.pressure.unwrap().clone()),
                visibility: parse_visibility(metar.visibility.unwrap().clone(), units),
                vert_visibility: parse_vert_visibility(metar.vert_visibility),
                remark: metar.remarks.unwrap(),
                station: metar.station,
                time,
            }
        }
        DisplayUnits::Nautical => {
            rep = WeatherReport {
                units,
                wind_speed: parse_speed_kts(metar.wind.speed.unwrap().clone()),
                wind_gust: match metar.wind.gusting {
                    None => None,
                    Some(speed) => Some(parse_speed_kts(speed.clone())),
                },
                wind_dir: parse_wind_dir(metar.wind.dir.unwrap().clone()),
                wind_status: parse_wind_status(metar.wind.dir.unwrap().clone()),
                temp: degrees_c_to_f(*metar.temperature.unwrap() as f64) as i32,
                dew_point: *metar.dewpoint.unwrap(),
                clouds: parse_clouds(metar.cloud_layers, units),
                pressure: parse_pressure_hpa(metar.pressure.unwrap().clone()).round(),
                visibility: parse_visibility(metar.visibility.unwrap().clone(), units),
                vert_visibility: parse_vert_visibility(metar.vert_visibility),
                remark: metar.remarks.unwrap(),
                station: metar.station,
                time,
            }
        }
        _ => {
            rep = WeatherReport {
                units,
                wind_speed: parse_speed_kts(metar.wind.speed.unwrap().clone()),
                wind_gust: match metar.wind.gusting {
                    None => None,
                    Some(speed) => Some(parse_speed_kts(speed.clone())),
                },
                wind_dir: parse_wind_dir(metar.wind.dir.unwrap().clone()),
                wind_status: parse_wind_status(metar.wind.dir.unwrap().clone()),
                temp: *metar.dewpoint.unwrap(),
                dew_point: *metar.temperature.unwrap(),
                clouds: parse_clouds(metar.cloud_layers, units),
                pressure: parse_pressure_hpa(metar.pressure.unwrap().clone()),
                visibility: parse_visibility(metar.visibility.unwrap().clone(), units),
                vert_visibility: parse_vert_visibility(metar.vert_visibility),
                remark: metar.remarks.unwrap(),
                station: metar.station,
                time,
            }
        }
    }
    rep
}

async fn download_files(url: &str, path: PathBuf) -> Result<(), Box<dyn std::error::Error>>  {
    use tokio::{
        io::{ AsyncWriteExt },
        fs::{ File },
    };
    let mut file = File::create(path).await?;
    // log::info!("Downloading {}...", url);
    let mut stream = reqwest::get(url)
        .await?
        .bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    log::info!("Downloaded {}", url);
    Ok(())
}
pub async fn download_metars(dir: PathBuf) -> PathBuf {
    let out_dir = dir.join("txtmin20");
    let zip = dir.join("txtmin20.zip");
    let mut download = true;
    if zip.exists() {
        let age = fs::metadata(zip.clone())
            .unwrap()
            .modified()
            .unwrap()
            .elapsed()
            .unwrap();
        log::info!("age: {:?}", age);
        if age.as_secs() > 60 * 20 {
            fs::remove_file(&zip).unwrap();
        } else {
            download = false;
        }
    }
    if download {
        log::info!("downloading latest data");
        let zip = dir.join("txtmin20.zip");
        download_files(
            "https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/txtmin20.zip",
            zip.clone(),
        )
        .await
        .unwrap();
    }
    Command::new("unzip")
        .args([
            zip.to_str().unwrap(),
            &*format!("-d{}", out_dir.to_str().unwrap()),
        ])
        .output()
        .unwrap();
    let l = fs::read_dir(out_dir.clone())
        .unwrap()
        .map(|f| f.unwrap())
        .filter(|x| x.path().extension().unwrap().eq_ignore_ascii_case("ZIP"))
        .collect::<Vec<DirEntry>>();
    for dir in l {
        Command::new("unzip")
            .args([dir.path().to_str().unwrap(), "-dtxtmin20"])
            .output()
            .unwrap();
    }
    out_dir
}
fn __parse_time(metar: String, icao: String) -> Option<DateTime<Utc>> {
    let mut str = metar
        .clone()
        .replace(icao.as_str(), "")
        .trim()
        .to_string()
        .replace("\n", "")
        .replace("\r", "");
    let _ = str.split_off(7);
    let days = str[..2].to_string().parse::<u32>();
    let hours = str[2..4].to_string().parse::<u32>();
    let minutes = str[4..6].to_string().parse::<u32>();
    if days.is_err() || hours.is_err() || minutes.is_err() {
        return None;
    }
    Some(
        Utc::now()
            .with_day(days.unwrap())
            .unwrap()
            .with_hour(hours.unwrap())
            .unwrap()
            .with_minute(minutes.unwrap())
            .unwrap(),
    )
}
fn max_idx(arr: Vec<DateTime<Utc>>) -> usize {
    let mut i = 0;
    for (j, &val) in arr.iter().enumerate() {
        if val > arr[j] {
            i = j;
        }
    }
    i
}

pub fn update_metar(
    path: PathBuf,
    icao: String,
    units: DisplayUnits,
    output_dir: PathBuf,
    write_brief: bool,
) -> Option<(String, String)> {
    let metars = get_metars(icao.to_string(), path);
    if metars.len() == 0 {
        println!(
            "No weather for {:} is available in the latest report",
            icao
        );
        return None;
    }
    let mut time = Vec::<DateTime<Utc>>::new();
    for metar in metars.clone() {
        let t = __parse_time(metar, icao.clone());
        if t.is_some() {
            time.push(t.unwrap());
        }
    }
    let latest = metars[max_idx(time.clone())].clone();
    let cur_metar = Metar::parse(latest).unwrap();
    let rep = write_report(cur_metar.clone(), units);
    let t = parse_time(cur_metar.clone());
    let filename = format!(
        "weather_{}_{}.toml",
        icao,
        t.to_rfc3339_opts(SecondsFormat::Secs, true)
    );
    let brief = parse_metar_brief(cur_metar, units);
    if write_brief {
        let file = format!(
            "brief_{icao}_{}.txt",
            t.to_rfc3339_opts(SecondsFormat::Secs, true)
        );
        File::create(output_dir.clone().join(file))
            .unwrap()
            .write_all(brief.as_bytes())
            .unwrap();
    }
    let toml_file = File::create(output_dir.clone().join(filename.clone()));
    toml_file
        .unwrap()
        .write_all(toml::to_string(&rep).unwrap().as_bytes())
        .unwrap();
    Some((filename, brief))
}

#[cfg(feature = "metar-bin")]
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_name = "dir")]
    dir: Option<String>,
    #[arg(short='u', long, value_name = "units")]
    units: String,
    #[arg(short, long, value_name = "icao")]
    icao: Option<String>,
    #[arg(short, long)]
    city_name: Option<String>,
    #[arg(short = 'b', long, value_name = "brief")]
    brief: bool,
    #[arg(short = 'w', long, value_name = "web")]
    use_web: bool,
    #[arg(short = 'o', long, value_name = "output")]
    output_dir: Option<String>,
}
#[cfg(feature = "metar-bin")]
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let units;
    match args.units.to_lowercase().as_str() {
        "metric" => units = DisplayUnits::Metric,
        "imperial" => units = DisplayUnits::Imperial,
        "nautical" => units = DisplayUnits::Nautical,
        "aviation" => units = DisplayUnits::Aviation,
        _ => {
            println!("invalid units given");
            return;
        }
    }
    if args.icao.is_none() && args.city_name.is_none() {
        println!("please enter a City Name or Airport ICAO code");
        return;
    }
    if args.icao.is_some() {
        println!("getting weather for {:}", args.icao.clone().unwrap());
        let icao = args.icao.clone().unwrap();
        let mut path = match args.dir.clone() {
            None => env::temp_dir().join("radio-station"),
            Some(p) => PathBuf::from(p),
        };

        if !path.clone().exists() {
            tokio::fs::create_dir(path.clone()).await.unwrap();
            // fs::create_dir(path.clone()).unwrap();
        }
        if args.use_web {
            path = download_metars(path).await;
        } else {
            path = PathBuf::from(&path);
        }
        let output_dir = match args.output_dir {
            None => path.clone(),
            Some(p) => PathBuf::from(p),
        };
        let res = update_metar(path, icao.clone(), units, output_dir, true);
        match res {
            None => {

            }
            Some((filename, brief)) => {
                println!("wrote {}", filename);
                println!("{}", brief);
                if args.brief {
                    println!("writing brief");
                }
            }
        }
    }
}
