mod weather_types;

use std::error::Error;
use ureq;

use serde_json;
pub use weather_types::OpenWeather;

pub fn get_weather(
    appid: &str,
    lat: &str,
    lon: &str,
    units: &str,
) -> Result<OpenWeather, Box<dyn Error>> {
    let uri = format!(
        "https://api.openweathermap.org/data/2.5/weather?appid={}&lat={}&lon={}&units={}",
        appid, lat, lon, units
    );

    let mut req = ureq::get(&uri);
    let resp = req.call();
    if resp.ok() {
        let weather = serde_json::from_str::<OpenWeather>(
            &resp.into_string().expect("failed to convert into string"),
        )
        .expect("failed to convert to type");

        Ok(weather)
    } else {
        panic!("bad response: {:?} - req: {:?}", resp, req);
    }
}
