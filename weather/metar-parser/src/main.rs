use chrono::{Datelike, SecondsFormat, Timelike, Utc};
use metar::{
    CloudLayer, CloudType, Metar, Pressure, VertVisibility, Visibility, WindDirection, WindSpeed,
};
use serde::Serialize;
use std::cmp::PartialEq;
use std::fs::{DirEntry, File};
use std::io::Write;
use std::process::Command;
use std::{env, fs};
use toml;
use std::str::FromStr;
use std::time::Duration;

fn degrees_c_to_f(temp: f64) -> f64 {
    temp * (9.0 / 5.0) + 32.0
}
fn kts_to_kph(speed: f64) -> f64 {
    speed * 1.852
}
fn kph_to_kts(speed: f64) -> f64 {
    speed * 0.53996
}
fn mps_to_kts(speed: f64) -> f64 {
    speed * 1.943844
}
fn mps_to_kph(speed: f64) -> f64 {
    speed * 3.6
}
fn ft_to_m(length: f64) -> f64 {
    length * 3.28084
}
fn m_to_mi(length: f64) -> f64 {
    length / 1609.344
}
fn mi_to_m(length: f64) -> f64 {
    length * 1609.344
}
fn parse_visibility(vis: Visibility, units: DisplayUnits) -> Option<f64> {
    if units != DisplayUnits::Metric {
        return match vis {
            Visibility::CAVOK => None,
            Visibility::Metres(m) => Option::from(m_to_mi(m as f64)),
            Visibility::StatuteMiles(mi) => Option::from(mi as f64),
        };
    } else {
        match vis {
            Visibility::CAVOK => None,
            Visibility::Metres(m) => Option::from(m_to_mi(m as f64)),
            Visibility::StatuteMiles(mi) => Option::from(f64::from(mi)),
        }
    }
}
fn inhg_to_hpa(press: f64) -> f64 {
    press * 33.863889532611
}
fn hpa_to_inhg(press: f64) -> f64 {
    press * 0.02952998057228
}
fn parse_speed_kts(windspeed: WindSpeed) -> f64 {
    match windspeed {
        WindSpeed::Calm => 0.0,
        WindSpeed::Knot(k) => f64::from(k),
        WindSpeed::MetresPerSecond(mps) => mps_to_kts(f64::from(mps)),
        WindSpeed::KilometresPerHour(kph) => kph_to_kts(f64::from(kph)),
    }
}
fn parse_speed_kph(windspeed: WindSpeed) -> f64 {
    match windspeed {
        WindSpeed::Calm => 0.0,
        WindSpeed::Knot(kts) => kts_to_kph(f64::from(kts)),
        WindSpeed::MetresPerSecond(mps) => mps_to_kph(f64::from(mps)),
        WindSpeed::KilometresPerHour(kph) => f64::from(kph),
    }
}

fn parse_pressure_hpa(press: Pressure) -> f64 {
    match press {
        Pressure::Hectopascals(hpa) => hpa as f64,
        Pressure::InchesOfMercury(inhg) => inhg_to_hpa(f64::from(inhg)),
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
        let cloudtype: CloudType;
        let cloud_height;
        match cloud.clone() {
            CloudLayer::Few(cl_type, height) => {
                s = "few ".to_string();
                cloudtype = cl_type;
                cloud_height = height;
            }
            CloudLayer::Scattered(cl_type, height) => {
                s = "scattered ".to_string();
                cloudtype = cl_type;
                cloud_height = height;
            }
            CloudLayer::Broken(cl_type, height) => {
                s = "broken ".to_string();
                cloudtype = cl_type;
                cloud_height = height;
            }
            CloudLayer::Overcast(cl_type, height) => {
                s = "overcast ".to_string();
                cloudtype = cl_type;
                cloud_height = height;
            }
            CloudLayer::Unknown(cl_type, height) => {
                s = "".to_string();
                cloudtype = cl_type;
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
            cloud_type: format!("{:?}", cloudtype),
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
    dewpoint: i32,
    clouds: Vec<CloudData>,
    pressure: f64,
    visibility: Option<f64>,
    vert_visibility: Option<u32>,
    remark: String,
    station: String,
    time: toml::value::Datetime,
}

#[derive(PartialEq, Eq, Debug, Serialize, Clone, Copy)]
enum DisplayUnits {
    Imperial,
    Metric,
    Aviation,
    Nautical,
}
fn get_metars(icao: String, dir_path: String) -> Vec<String> {
    let icao = icao.as_str();
    let mut metars = Vec::new();
    let dir = fs::read_dir(dir_path).unwrap();
    for filepath in dir {
        let fname = filepath.unwrap().path();
        if !fname.extension().unwrap().eq_ignore_ascii_case("txt") {
            continue;
        }
        let text = fs::read_to_string(fname).unwrap();
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
fn parse_metar(cur_metar: Metar, units: DisplayUnits) -> String {
    let mut report = String::with_capacity(50);
    if units == DisplayUnits::Nautical || units == DisplayUnits::Imperial {
        report += format!(
            "{:.0}ºF ",
            degrees_c_to_f(f64::from(*cur_metar.temperature.unwrap()))
        )
        .as_str();
    } else {
        report += format!("{:.0}ºC ", f64::from(*cur_metar.temperature.unwrap())).as_str();
    }
    if cur_metar.weather.len() == 0 {
        report += "skies clear "
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

fn write_report(metar: Metar, units: DisplayUnits) -> WeatherReport {
    let t = Utc::now()
        .with_day(metar.time.date as u32)
        .unwrap()
        .with_hour(metar.time.hour as u32)
        .unwrap()
        .with_second(0)
        .unwrap();
    let time = toml::value::Datetime::from_str(t.to_rfc3339_opts(SecondsFormat::Secs, true).as_str()).unwrap();

    let rep;
    match units {

        DisplayUnits::Metric => {
            rep = WeatherReport{
                units,
                wind_speed: parse_speed_kph(metar.wind.speed.unwrap().clone()),
                wind_gust: match metar.wind.gusting {
                    None => None,
                    Some(speed)=>Some(parse_speed_kph(speed))
                },
                wind_dir: parse_wind_dir(metar.wind.dir.unwrap().clone()),
                wind_status: parse_wind_status(metar.wind.dir.unwrap().clone()),
                temp: *metar.temperature.unwrap(),
                dewpoint: *metar.temperature.unwrap(),
                clouds: parse_clouds(metar.cloud_layers, units),
                pressure: parse_pressure_hpa(metar.pressure.unwrap().clone()),
                visibility: parse_visibility(metar.visibility.unwrap().clone(), units),
                vert_visibility: match parse_vert_visibility(metar.vert_visibility) {
                    Some(vis) => Some(ft_to_m(vis as f64) as u32),
                    None => None
                },
                remark: metar.remarks.unwrap(),
                station: metar.station,
                time:time
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
                dewpoint: *metar.temperature.unwrap(),
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
                dewpoint: *metar.dewpoint.unwrap(),
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
            rep=WeatherReport{
                units,
                wind_speed: parse_speed_kts(metar.wind.speed.unwrap().clone()),
                wind_gust: Option::from(parse_speed_kts(metar.wind.speed.unwrap().clone())),
                wind_dir: parse_wind_dir(metar.wind.dir.unwrap().clone()),
                wind_status: parse_wind_status(metar.wind.dir.unwrap().clone()),
                temp: *metar.dewpoint.unwrap(),
                dewpoint: *metar.temperature.unwrap(),
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
fn main() {

    let icao = "KJFK";
    let mut p = env::current_dir()
        .unwrap()
        .join("txtmin20")
        .to_str()
        .unwrap()
        .to_string();
    let age = fs::metadata(&p).unwrap().modified().unwrap().elapsed().unwrap();
    if  age > Duration::from_secs(60 * 20) {
        print!("downloading latest data");
        Command::new("rm").args(&["-rf", "txtmin20.zip"]);
        Command::new("wget")
            .arg("https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/txtmin20.zip")
            .output()
            .unwrap();
        Command::new("unzip")
            .args(["txtmin20.zip", "-dtxtmin20"])
            .output()
            .unwrap();

    }

    let l = fs::read_dir(p.clone())
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
    p = env::current_dir()
        .unwrap()
        .join("txtmin20")
        .to_str()
        .unwrap()
        .to_string();
    let metars = get_metars(icao.to_string(), p);
    if metars.len() == 0 {
        return;
    }
    let cur_metar = Metar::parse(metars[0].to_string()).unwrap();
    let rep = write_report(cur_metar.clone(), DisplayUnits::Aviation);
    let t = Utc::now()
        .with_day(cur_metar.time.date as u32)
        .unwrap()
        .with_hour(cur_metar.time.hour as u32)
        .unwrap().with_minute(cur_metar.time.minute as u32)
        .unwrap()
        .with_second(0)
        .unwrap();
    let filename = "weather_".to_string()
        + icao
        + "_"
        + t.to_rfc3339_opts(SecondsFormat::Secs, true).as_str()
        + ".toml";
    println!("{}", filename);
    let toml_file = File::create(filename);
    toml_file
        .unwrap()
        .write_all(toml::to_string(&rep).unwrap().as_bytes())
        .unwrap();
    println!("{}", parse_metar(cur_metar, DisplayUnits::Aviation));
}
