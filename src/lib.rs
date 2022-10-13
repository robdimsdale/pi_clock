mod config;
mod display;
mod light;
mod weather;

use chrono::{Local, Timelike};
pub use config::Config;
#[cfg(feature = "rpi-hw")]
pub use display::{AlphaNum4Display, LCD16x2Display, LCD20x4Display, SevenSegment4Display};
pub use display::{Console16x2Display, Console20x4Display, Display, DisplayType};
#[cfg(feature = "rpi-hw")]
pub use light::VEML7700LightSensor;
pub use light::{LightSensor, LightSensorType, RandomLightSensor, TimeLightSensor};
use log::{info, warn};
use std::collections::HashMap;
use std::fmt;
use std::{thread, time};
pub use weather::OpenWeather;

const STATE_COUNT: u32 = 3;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl std::error::Error for Error {}

impl Error {
    /// Return the kind of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

/// The kind of an error that can occur.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    Weather(Box<weather::Error>),
    Display(display::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Weather(ref err) => err.fmt(f),
            ErrorKind::Display(ref err) => err.fmt(f),
        }
    }
}

impl From<weather::Error> for Error {
    fn from(e: weather::Error) -> Self {
        Error {
            kind: ErrorKind::Weather(Box::new(e)),
        }
    }
}

impl From<display::Error> for Error {
    fn from(e: display::Error) -> Self {
        Error {
            kind: ErrorKind::Display(e),
        }
    }
}

pub fn run<T: LightSensor>(
    config: &Config,
    display: &mut display::DisplayType<T>,
) -> Result<(), Error> {
    let no_weather_error_duration = config.weather_request_polling_interval * 3;

    let state_machine = StateMachine::new(STATE_COUNT, config.state_duration.as_secs() as u32);

    let mut last_weather_attempt = time::Instant::now();
    let mut last_weather_success = time::Instant::now();

    let mut weather = match weather::get_weather(&config.uri, config.weather_request_timeout) {
        Ok(w) => Some(w),
        Err(e) => {
            warn!("Error getting initial weather: {}", e);
            None
        }
    };

    loop {
        let now = time::Instant::now();

        let duration_since_last_weather = now.duration_since(last_weather_attempt);
        if duration_since_last_weather > config.weather_request_polling_interval {
            last_weather_attempt = now;

            info!(
                "Getting updated weather ({}s since last attempt)",
                config.weather_request_polling_interval.as_secs(),
            );

            match weather::get_weather(&config.uri, config.weather_request_timeout) {
                Ok(updated_weather) => {
                    info!("successfully updated weather");

                    last_weather_success = now;
                    weather = Some(updated_weather)
                }
                Err(e) => {
                    warn!(
                        "Error updating weather: {}. Using previous weather. {}s since last success", e,
                        now.duration_since(last_weather_success).as_secs()
                    );
                }
            };
        }

        if now > last_weather_success + no_weather_error_duration {
            warn!(
                "no successful weather in over {}s. Displaying empty weather",
                no_weather_error_duration.as_secs()
            );
            display.print(&Local::now(), state_machine.current_state(), &None)?;
        } else {
            display.print(&Local::now(), state_machine.current_state(), &weather)?;
        }

        thread::sleep(config.loop_sleep_duration);
    }
}

struct StateMachine {
    state_map: HashMap<u32, u32>,
    state_count: u32,
    state_duration_secs: u32,
}

impl StateMachine {
    fn new(state_count: u32, state_duration_secs: u32) -> Self {
        let mut state_map = HashMap::new();

        let mut current_build_state = 0;
        for i in 0..state_duration_secs * state_count {
            state_map.insert(i, current_build_state);
            if (i + 1) % state_duration_secs == 0 {
                current_build_state += 1;
            }
        }

        StateMachine {
            state_map,
            state_duration_secs,
            state_count,
        }
    }

    fn current_state(&self) -> u32 {
        let second_mod = Local::now().second() % (self.state_duration_secs * self.state_count);
        *self.state_map.get(&second_mod).unwrap()
    }
}
