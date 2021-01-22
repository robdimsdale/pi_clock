mod error;
mod open_weather_types;
mod units;

pub use error::Error;
pub use open_weather_types::OpenWeather;
use std::time::Duration;
pub use units::TemperatureUnits;

pub fn get_weather(
    appid: &str,
    lat: &str,
    lon: &str,
    units: &TemperatureUnits,
) -> Result<OpenWeather, Error> {
    let uri = format!(
        "https://api.openweathermap.org/data/2.5/weather?appid={}&lat={}&lon={}&units={}",
        appid,
        lat,
        lon,
        units.to_string()
    );

    let agent = ureq::builder().timeout(Duration::from_millis(500)).build();

    let response = agent.get(&uri).call()?.into_string()?;

    Ok(serde_json::from_str(&response)?)
}
