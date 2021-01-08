mod open_weather_types;

use std::error::Error;
pub use open_weather_types::OpenWeather;

pub const UNITS_IMPERIAL: &'static str = "imperial";
pub const UNITS_METRIC: &'static str = "metric";
pub const UNITS_STANDARD: &'static str = "standard";

pub enum Units {
    Imperial,
    Metric,
    Standard,
}

impl ToString for Units {
    fn to_string(&self) -> String {
        match self {
            Self::Imperial => UNITS_IMPERIAL.to_owned(),
            Self::Metric => UNITS_METRIC.to_owned(),
            Self::Standard => UNITS_STANDARD.to_owned(),
        }
    }
}

impl Units {
    pub fn from_string(s: &str) -> Units {
        match s {
            UNITS_IMPERIAL => Units::Imperial,
            UNITS_METRIC => Units::Metric,
            UNITS_STANDARD => Units::Standard,
            _ => panic!("Unrecognized units: {}", s),
        }
    }
}

pub fn get_weather(
    appid: &str,
    lat: &str,
    lon: &str,
    units: &Units,
) -> Result<open_weather_types::OpenWeather, Box<dyn Error>> {
    let uri = format!(
        "https://api.openweathermap.org/data/2.5/weather?appid={}&lat={}&lon={}&units={}",
        appid,
        lat,
        lon,
        units.to_string()
    );

    let mut req = ureq::get(&uri);
    let resp = req.call();
    if resp.ok() {
        let weather = serde_json::from_str::<open_weather_types::OpenWeather>(
            &resp.into_string().expect("failed to convert into string"),
        )
        .expect("failed to convert to type");

        Ok(weather)
    } else {
        panic!("bad response: {:?} - req: {:?}", resp, req);
    }
}
