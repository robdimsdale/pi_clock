mod error;
mod open_weather_types;

use chrono::{DateTime, Local, TimeZone};
pub use error::Error;
pub use open_weather_types::{Main, OpenWeather};
use std::time::Duration;

pub fn get_weather(uri: &str, timeout: Duration) -> Result<OpenWeather, Error> {
    let agent = ureq::builder().timeout(timeout).build();

    let response = agent.get(uri).call()?.into_string()?;

    Ok(serde_json::from_str(&response)?)
}

fn timestamp_before_now(ts: &DateTime<Local>) -> bool {
    *ts - Local::now() < chrono::Duration::zero()
}

fn timestamp_after_24_hours(ts: &DateTime<Local>) -> bool {
    *ts - Local::now() > chrono::Duration::hours(24)
}

pub fn currently_raining(w: &OpenWeather) -> bool {
    w.current.weather[0].main == Main::Rain
}

pub fn high_low_temp(w: &OpenWeather) -> ((DateTime<Local>, f32), (DateTime<Local>, f32)) {
    let mut high = &w.hourly[0];
    let mut low = &w.hourly[0];

    for h in w.hourly.iter() {
        let ts = Local.timestamp(h.dt, 0);
        if timestamp_before_now(&ts) {
            continue;
        }

        if timestamp_after_24_hours(&ts) {
            continue;
        }

        if h.temp > high.temp {
            high = h
        }

        if h.temp < low.temp {
            low = h
        }
    }

    (
        (Local.timestamp(high.dt, 0), high.temp),
        (Local.timestamp(low.dt, 0), low.temp),
    )
}

// Returns the next time that the rain is forecast to
// start (if it is not currently raining)
// or stop (if it is currently raining)
// If the rain will not change within the next 24 hours, no time is returned.
pub fn next_rain_start_or_stop(w: &OpenWeather) -> Option<DateTime<Local>> {
    for h in w.hourly.iter() {
        let ts = Local.timestamp(h.dt, 0);
        if timestamp_before_now(&ts) {
            continue;
        }

        if timestamp_after_24_hours(&ts) {
            return None;
        }

        if currently_raining(w) && h.weather[0].main != Main::Rain {
            return Some(ts);
        }

        if !currently_raining(w) && h.weather[0].main == Main::Rain {
            return Some(ts);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::open_weather_types::Weather;

    #[test]
    fn test_next_rain_start() {
        let mut w: OpenWeather = Default::default();
        w.current.weather = vec![Weather {
            id: 1234,
            main: Main::Rain,
            description: "Light Rain".to_string(),
            icon: "some-icon".to_string(),
        }];

        w.hourly = vec![Default::default(), Default::default(), Default::default()];

        w.hourly[0].dt = (Local::now() - chrono::Duration::minutes(30)).timestamp();
        w.hourly[1].dt = (Local::now() + chrono::Duration::hours(1)).timestamp();
        w.hourly[2].dt = (Local::now() + chrono::Duration::hours(2)).timestamp();

        w.hourly[0].weather = vec![Weather {
            id: 2345,
            main: Main::Rain,
            description: "Light Rain".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Rain,
            description: "Light Rain".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_rain_start_or_stop(&w);

        assert_eq!(maybe_next_change, Some(Local.timestamp(w.hourly[2].dt, 0)));
    }

    #[test]
    fn test_next_rain_stop() {
        let mut w: OpenWeather = Default::default();
        w.current.weather = vec![Weather {
            id: 1234,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        w.hourly = vec![Default::default(), Default::default(), Default::default()];

        w.hourly[0].dt = (Local::now() - chrono::Duration::minutes(30)).timestamp();
        w.hourly[1].dt = (Local::now() + chrono::Duration::hours(1)).timestamp();
        w.hourly[2].dt = (Local::now() + chrono::Duration::hours(2)).timestamp();

        w.hourly[0].weather = vec![Weather {
            id: 2345,
            main: Main::Rain,
            description: "Light Rain".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Rain,
            description: "Light Rain".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_rain_start_or_stop(&w);

        assert_eq!(maybe_next_change, Some(Local.timestamp(w.hourly[1].dt, 0)));
    }

    #[test]
    fn test_next_rain_never_starts() {
        let mut w: OpenWeather = Default::default();
        w.current.weather = vec![Weather {
            id: 1234,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        w.hourly = vec![Default::default(), Default::default(), Default::default()];

        w.hourly[0].dt = (Local::now() - chrono::Duration::minutes(30)).timestamp();
        w.hourly[1].dt = (Local::now() + chrono::Duration::hours(1)).timestamp();
        w.hourly[2].dt = (Local::now() + chrono::Duration::hours(2)).timestamp();

        w.hourly[0].weather = vec![Weather {
            id: 2345,
            main: Main::Rain,
            description: "Light Rain".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_rain_start_or_stop(&w);

        assert_eq!(maybe_next_change, None);
    }

    #[test]
    fn test_next_rain_never_stops() {
        let mut w: OpenWeather = Default::default();
        w.current.weather = vec![Weather {
            id: 1234,
            main: Main::Rain,
            description: "Rain".to_string(),
            icon: "some-icon".to_string(),
        }];

        w.hourly = vec![Default::default(), Default::default(), Default::default()];

        w.hourly[0].dt = (Local::now() - chrono::Duration::minutes(30)).timestamp();
        w.hourly[1].dt = (Local::now() + chrono::Duration::hours(1)).timestamp();
        w.hourly[2].dt = (Local::now() + chrono::Duration::hours(2)).timestamp();

        w.hourly[0].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Rain,
            description: "Light Rain".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Rain,
            description: "Rain".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_rain_start_or_stop(&w);

        assert_eq!(maybe_next_change, None);
    }

    #[test]
    fn test_next_rain_no_change_in_24_hours() {
        let mut w: OpenWeather = Default::default();
        w.current.weather = vec![Weather {
            id: 1234,
            main: Main::Rain,
            description: "Rain".to_string(),
            icon: "some-icon".to_string(),
        }];

        w.hourly = vec![Default::default(), Default::default(), Default::default()];

        w.hourly[0].dt = (Local::now() - chrono::Duration::minutes(30)).timestamp();
        w.hourly[1].dt = (Local::now() + chrono::Duration::hours(1)).timestamp();
        w.hourly[2].dt =
            (Local::now() + chrono::Duration::hours(24) + chrono::Duration::minutes(1)).timestamp(); // Add one minute to avoid racy tests.

        w.hourly[0].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Rain,
            description: "Light Rain".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_rain_start_or_stop(&w);

        assert_eq!(maybe_next_change, None);
    }
}
