mod display;
mod light;
mod weather;

use chrono::{Local, Timelike};
#[cfg(target_arch = "arm")]
pub use display::{
    AlphaNum4Display, ILI9341Display, LCD16x2Display, LCD20x4Display, SevenSegment4Display,
};
pub use display::{Console16x2Display, Console20x4Display, Display, DisplayType};
#[cfg(target_arch = "arm")]
pub use light::VEML7700LightSensor;
pub use light::{LightSensor, LightSensorType, RandomLightSensor, TimeLightSensor};
use log::{info, warn};
use std::collections::HashMap;
use std::fmt;
use std::{thread, time};
pub use weather::OpenWeather;

const WEATHER_DURATION_SECONDS: u64 = 60;
const NO_WEATHER_ERROR_DURATION_SECONDS: u64 = 3 * WEATHER_DURATION_SECONDS;

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
pub enum ErrorKind {
    Weather(weather::Error),
    Display(display::Error),
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Weather(ref err) => err.fmt(f),
            ErrorKind::Display(ref err) => err.fmt(f),
            ErrorKind::__Nonexhaustive => unreachable!(),
        }
    }
}

impl From<weather::Error> for Error {
    fn from(e: weather::Error) -> Self {
        Error {
            kind: ErrorKind::Weather(e),
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
    uri: &str,
    sleep_duration_millis: u64,
    state_duration_secs: u32,
    display: &mut display::DisplayType<T>,
) -> Result<(), Error> {
    let mut last_weather_attempt = time::Instant::now();
    let mut last_weather_success = time::Instant::now();

    let mut weather = match weather::get_weather(&uri) {
        Ok(w) => Some(w),
        Err(e) => {
            warn!("Error getting initial weather: {}", e);
            None
        }
    };

    loop {
        let now = time::Instant::now();

        let duration_since_last_weather = now.duration_since(last_weather_attempt);
        if duration_since_last_weather > time::Duration::from_secs(WEATHER_DURATION_SECONDS) {
            last_weather_attempt = now;

            info!(
                "Getting updated weather ({}s since last attempt)",
                WEATHER_DURATION_SECONDS,
            );

            match weather::get_weather(&uri) {
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

        if now > last_weather_success + time::Duration::from_secs(NO_WEATHER_ERROR_DURATION_SECONDS)
        {
            warn!(
                "no successful weather in over {}s. Displaying empty weather",
                NO_WEATHER_ERROR_DURATION_SECONDS
            );
            display.print(&Local::now(), current_state(state_duration_secs), &None)?;
        } else {
            display.print(&Local::now(), current_state(state_duration_secs), &weather)?;
        }

        thread::sleep(time::Duration::from_millis(sleep_duration_millis));
    }
}

// TODO: Make this an object with a current_state method on it.
// Or at the minimum, at least memoize it, so we don't have to pass state_duration_secs around
// every time we want to get the current state.
// We should not be building the map on every iteration.
fn state_map(state_duration_secs: u32) -> HashMap<u32, u32> {
    let mut state_map = HashMap::new();

    let mut current_build_state = 0;
    for i in 0..state_duration_secs * STATE_COUNT {
        state_map.insert(i, current_build_state);
        if (i + 1) % state_duration_secs == 0 {
            current_build_state += 1;
        }
    }

    state_map
}

fn current_state(state_duration_secs: u32) -> u32 {
    let second_mod = Local::now().second() % (state_duration_secs * STATE_COUNT);
    *state_map(state_duration_secs).get(&second_mod).unwrap()
}
