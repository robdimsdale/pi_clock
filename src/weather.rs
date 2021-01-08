mod open_weather_types;
mod units;

use std::error::Error;
pub use open_weather_types::OpenWeather;
pub use units::TemperatureUnits;

pub fn get_weather(
    appid: &str,
    lat: &str,
    lon: &str,
    units: &TemperatureUnits,
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
