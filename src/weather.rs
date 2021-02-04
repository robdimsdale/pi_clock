mod error;
mod open_weather_types;

pub use error::Error;
pub use open_weather_types::OpenWeather;
use std::time::Duration;

const TIMEOUT_MILLIS: u64 = 500;

pub fn get_weather(uri: &str) -> Result<OpenWeather, Error> {
    let agent = ureq::builder()
        .timeout(Duration::from_millis(TIMEOUT_MILLIS))
        .build();

    let response = agent.get(&uri).call()?.into_string()?;

    Ok(serde_json::from_str(&response)?)
}
