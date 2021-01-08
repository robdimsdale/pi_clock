use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Coord {
    pub lat: f32,
    pub lon: f32,
}

impl Default for Coord {
    fn default() -> Self {
        Coord {
            lat: f32::default(),
            lon: f32::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Description {
    pub id: i32,
    pub main: String,
    pub description: String,
    pub icon: String,
}

impl Default for Description {
    fn default() -> Self {
        Description {
            id: i32::default(),
            main: String::default(),
            description: String::default(),
            icon: String::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
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

impl Default for Main {
    fn default() -> Self {
        Main {
            temp: f32::default(),
            feels_like: f32::default(),
            temp_min: f32::default(),
            temp_max: f32::default(),
            pressure: f32::default(),
            humidity: f32::default(),
            sea_level: f32::default(),
            grnd_level: f32::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Wind {
    pub speed: f32,
    pub deg: f32,
    pub gust: f32,
}

impl Default for Wind {
    fn default() -> Self {
        Wind {
            speed: f32::default(),
            deg: f32::default(),
            gust: f32::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Precipitation {
    #[serde(rename = "1h")]
    pub one_hour: f32,
    #[serde(rename = "3h")]
    pub three_hour: f32,
}

impl Default for Precipitation {
    fn default() -> Self {
        Precipitation {
            one_hour: f32::default(),
            three_hour: f32::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Clouds {
    pub all: i32,
}

impl Default for Clouds {
    fn default() -> Self {
        Clouds {
            all: i32::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Sys {
    pub r#type: i32,
    pub id: i32,
    pub country: String,
    pub sunrise: i64,
    pub sunset: i64,
}

impl Default for Sys {
    fn default() -> Self {
        Sys {
            r#type: i32::default(),
            id: i32::default(),
            country: String::default(),
            sunrise: i64::default(),
            sunset: i64::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
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

impl Default for OpenWeather {
    fn default() -> Self {
        OpenWeather {
            coord: Coord::default(),
            weather: vec![Description::default()],
            base: String::default(),
            main: Main::default(),
            visibility: f32::default(),
            wind: Wind::default(),
            rain: Precipitation::default(),
            snow: Precipitation::default(),
            clouds: Clouds::default(),
            dt: i64::default(),
            sys: Sys::default(),
            timezone: f32::default(),
            id: i32::default(),
            name: String::default(),
            cod: i32::default(),
        }
    }
}
