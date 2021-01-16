mod display;
mod light;
mod weather;

use chrono::Local;
#[cfg(target_arch = "arm")]
pub use display::{AlphaNum4Display, HD44780Display, ILI9341Display, SevenSegment4Display};
pub use display::{ConsoleDisplay, Display, DisplayType};
#[cfg(target_arch = "arm")]
pub use light::VEML7700LightSensor;
pub use light::{LightSensor, LightSensorType, RandomLightSensor, TimeLightSensor};
use std::{thread, time};
pub use weather::{OpenWeather, TemperatureUnits};

const SLEEP_DURATION_MILLIS: u64 = 100;
const WEATHER_DURATION_SECONDS: u64 = 600;

pub fn run<T: LightSensor>(
    open_weather_api_key: &str,
    lat: &str,
    lon: &str,
    units: &TemperatureUnits,
    display: &mut display::DisplayType<T>,
) {
    let mut last_weather_attempt = time::Instant::now();
    let mut last_weather_success = time::Instant::now();

    let mut weather = weather::get_weather(&open_weather_api_key, &lat, &lon, &units)
        .expect("failed to get initial weather");

    loop {
        let now = time::Instant::now();

        let duration_since_last_weather = now.duration_since(last_weather_attempt);
        if duration_since_last_weather > time::Duration::from_secs(WEATHER_DURATION_SECONDS) {
            last_weather_attempt = now;

            println!(
                "Getting updated weather ({}s since last attempt)",
                WEATHER_DURATION_SECONDS,
            );

            if let Ok(updated_weather) =
                weather::get_weather(&open_weather_api_key, &lat, &lon, &units)
            {
                println!("successfully updated weather");

                last_weather_success = last_weather_attempt;
                weather = updated_weather
            } else {
                println!(
                    "failed to update weather (using previous weather). {}s since last success",
                    now.duration_since(last_weather_success).as_secs()
                );
            }
        }

        display.print(&Local::now(), &weather, &units);

        thread::sleep(time::Duration::from_millis(SLEEP_DURATION_MILLIS));
    }
}
