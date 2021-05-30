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
    let state_machine = StateMachine::new(STATE_COUNT, state_duration_secs);

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
            display.print(&Local::now(), state_machine.current_state(), &None)?;
        } else {
            display.print(&Local::now(), state_machine.current_state(), &weather)?;
        }

        thread::sleep(time::Duration::from_millis(sleep_duration_millis));
    }
}

struct StateMachine {
    state_map: HashMap<u32, u32>,
    state_count: u32,
    state_duration_secs: u32,
}

impl StateMachine {
    fn new(state_count: u32, state_duration_secs: u32) -> Self {
        let mut map = HashMap::new();

        let mut current_build_state = 0;
        for i in 0..state_duration_secs * state_count {
            map.insert(i, current_build_state);
            if (i + 1) % state_duration_secs == 0 {
                current_build_state += 1;
            }
        }

        StateMachine {
            state_map: map,
            state_duration_secs: state_duration_secs,
            state_count: state_count,
        }
    }

    fn current_state(self: &Self) -> u32 {
        let second_mod = Local::now().second() % (self.state_duration_secs * self.state_count);
        *self.state_map.get(&second_mod).unwrap()
    }
}
