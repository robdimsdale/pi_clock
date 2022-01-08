use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenWeather {
    pub lat: f32,
    pub lon: f32,
    pub timezone: String,
    pub timezone_offset: f32,
    pub current: Current,
    pub minutely: Vec<Minutely>,
    pub hourly: Vec<Hourly>,
    pub daily: Vec<Daily>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Current {
    pub dt: i64,
    pub sunrise: i64,
    pub sunset: i64,
    pub temp: f32,
    pub feels_like: f32,
    pub pressure: f32,
    pub humidity: f32,
    pub dew_point: f32,
    pub uvi: f32,
    pub clouds: i32,
    pub visibility: i32,
    pub wind_speed: f32,
    pub wind_deg: f32,
    pub wind_gust: f32,
    pub rain: Rain,
    pub snow: Snow,
    pub weather: Vec<Weather>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Rain {
    #[serde(rename = "1h")]
    pub one_hour: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Snow {
    #[serde(rename = "1h")]
    pub one_hour: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Weather {
    pub id: i32,
    pub main: Main,
    pub description: String,
    pub icon: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Main {
    Thunderstorm,
    Drizzle,
    Rain,
    Snow,
    Mist,
    Smoke,
    Haze,
    Dust,
    Fog,
    Sand,
    Ash,
    Squall,
    Tornado,
    Clear,
    Clouds,
}

impl Default for Main {
    fn default() -> Main {
        Main::Clear
    }
}

impl fmt::Display for Main {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Main::Thunderstorm => write!(f, "Thunderstorm"),
            Main::Drizzle => write!(f, "Drizzle"),
            Main::Rain => write!(f, "Rain"),
            Main::Snow => write!(f, "Snow"),
            Main::Mist => write!(f, "Mist"),
            Main::Smoke => write!(f, "Smoke"),
            Main::Haze => write!(f, "Haze"),
            Main::Dust => write!(f, "Dust"),
            Main::Fog => write!(f, "Fog"),
            Main::Sand => write!(f, "Sand"),
            Main::Ash => write!(f, "Ash"),
            Main::Squall => write!(f, "Squall"),
            Main::Tornado => write!(f, "Tornado"),
            Main::Clear => write!(f, "Clear"),
            Main::Clouds => write!(f, "Clouds"),
        }
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Minutely {
    pub dt: i64,
    pub precipitation: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Hourly {
    pub dt: i64,
    pub temp: f32,
    pub feels_like: f32,
    pub pressure: f32,
    pub humidity: f32,
    pub dew_point: f32,
    pub uvi: f32,
    pub clouds: i32,
    pub visibility: i32,
    pub wind_speed: f32,
    pub wind_deg: f32,
    pub wind_gust: f32,
    pub pop: f32,
    pub rain: Rain,
    pub snow: Snow,
    pub weather: Vec<Weather>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Daily {
    pub dt: i64,
    pub sunrise: i64,
    pub sunset: i64,
    pub moonrise: i64,
    pub moonset: i64,
    pub moonphase: f32,
    pub temp: Temp,
    pub feels_like: FeelsLike,
    pub pressure: f32,
    pub humidity: f32,
    pub dew_point: f32,
    pub uvi: f32,
    pub pop: f32,
    pub clouds: i32,
    pub wind_speed: f32,
    pub wind_deg: f32,
    pub wind_gust: f32,
    pub rain: f32,
    pub snow: f32,
    pub weather: Vec<Weather>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Temp {
    pub morn: f32,
    pub day: f32,
    pub eve: f32,
    pub night: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct FeelsLike {
    pub morn: f32,
    pub day: f32,
    pub eve: f32,
    pub night: f32,
}
