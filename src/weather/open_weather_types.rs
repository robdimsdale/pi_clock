use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Coord {
    pub lat: f32,
    pub lon: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Description {
    pub id: i32,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Main {
    pub temp: f32,
    pub feels_like: f32,
    pub temp_min: f32,
    pub temp_max: f32,
    pub pressure: f32,
    pub humidity: f32,
    pub sea_level: f32,
    pub grnd_level: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Wind {
    pub speed: f32,
    pub deg: f32,
    pub gust: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Precipitation {
    #[serde(rename = "1h")]
    pub one_hour: f32,
    #[serde(rename = "3h")]
    pub three_hour: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Clouds {
    pub all: i32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Sys {
    pub r#type: i32,
    pub id: i32,
    pub country: String,
    pub sunrise: i64,
    pub sunset: i64,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenWeather {
    pub coord: Coord,
    pub weather: Vec<Description>,
    pub base: String,
    pub main: Main,
    pub visibility: f32,
    pub wind: Wind,
    pub rain: Precipitation,
    pub snow: Precipitation,
    pub clouds: Clouds,
    pub dt: i64,
    pub sys: Sys,
    pub timezone: f32,
    pub id: i32,
    pub name: String,
    pub cod: i32,
}
