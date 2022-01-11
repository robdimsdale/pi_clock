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

// TODO: add tests for PrecipitationChanges for types beyond rain
fn is_precipitation(w: Main) -> bool {
    matches!(
        w,
        Main::Rain | Main::Snow | Main::Drizzle | Main::Thunderstorm
    )
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PrecipitationChange {
    Start(DateTime<Local>, Main),
    Stop(DateTime<Local>, Main),
    NoChange(Option<Main>),
}

// Returns the next time that precipitation is forecast
// to start (if it is not currently precipitating)
// or to stop (if it is currently precipitating)
// If the precipitation changes between multiple types, all precipitation is assumed to be that
// type.
// e.g. If it is currently raining, then it snows, then it stops snowing, only the stop time
// is returned, and the precipitation change type is rain.
pub fn next_precipitation_change(w: &OpenWeather) -> PrecipitationChange {
    let current_precipitation = if is_precipitation(w.current.weather[0].main) {
        Some(w.current.weather[0].main)
    } else {
        None
    };

    for h in w.hourly.iter() {
        let ts = Local.timestamp(h.dt, 0);
        if timestamp_before_now(&ts) {
            continue;
        }

        if timestamp_after_24_hours(&ts) {
            return PrecipitationChange::NoChange(current_precipitation);
        }

        match current_precipitation {
            Some(p) => {
                if !is_precipitation(h.weather[0].main) {
                    return PrecipitationChange::Stop(ts, p);
                }
            }
            None => {
                if is_precipitation(h.weather[0].main) {
                    return PrecipitationChange::Start(ts, h.weather[0].main);
                }
            }
        }
    }

    PrecipitationChange::NoChange(current_precipitation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::open_weather_types::Weather;

    #[test]
    fn test_next_rain_stop() {
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

        let maybe_next_change = next_precipitation_change(&w);
        let expected = PrecipitationChange::Stop(Local.timestamp(w.hourly[2].dt, 0), Main::Rain);

        assert_eq!(maybe_next_change, expected);
    }

    #[test]
    fn test_next_rain_start() {
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

        let maybe_next_change = next_precipitation_change(&w);
        let expected = PrecipitationChange::Start(Local.timestamp(w.hourly[1].dt, 0), Main::Rain);

        assert_eq!(maybe_next_change, expected)
    }

    #[test]
    fn test_next_snow_stop() {
        let mut w: OpenWeather = Default::default();
        w.current.weather = vec![Weather {
            id: 1234,
            main: Main::Snow,
            description: "Light Snow".to_string(),
            icon: "some-icon".to_string(),
        }];

        w.hourly = vec![Default::default(), Default::default(), Default::default()];

        w.hourly[0].dt = (Local::now() - chrono::Duration::minutes(30)).timestamp();
        w.hourly[1].dt = (Local::now() + chrono::Duration::hours(1)).timestamp();
        w.hourly[2].dt = (Local::now() + chrono::Duration::hours(2)).timestamp();

        w.hourly[0].weather = vec![Weather {
            id: 2345,
            main: Main::Snow,
            description: "Light Snow".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Snow,
            description: "Light Snow".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_precipitation_change(&w);
        let expected = PrecipitationChange::Stop(Local.timestamp(w.hourly[2].dt, 0), Main::Snow);

        assert_eq!(maybe_next_change, expected);
    }

    #[test]
    fn test_next_snow_start() {
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
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Snow,
            description: "Light Snow".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_precipitation_change(&w);
        let expected = PrecipitationChange::Start(Local.timestamp(w.hourly[1].dt, 0), Main::Snow);

        assert_eq!(maybe_next_change, expected)
    }

    #[test]
    fn test_next_drizzle_stop() {
        let mut w: OpenWeather = Default::default();
        w.current.weather = vec![Weather {
            id: 1234,
            main: Main::Drizzle,
            description: "Light Drizzle".to_string(),
            icon: "some-icon".to_string(),
        }];

        w.hourly = vec![Default::default(), Default::default(), Default::default()];

        w.hourly[0].dt = (Local::now() - chrono::Duration::minutes(30)).timestamp();
        w.hourly[1].dt = (Local::now() + chrono::Duration::hours(1)).timestamp();
        w.hourly[2].dt = (Local::now() + chrono::Duration::hours(2)).timestamp();

        w.hourly[0].weather = vec![Weather {
            id: 2345,
            main: Main::Drizzle,
            description: "Light Drizzle".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Drizzle,
            description: "Light Drizzle".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_precipitation_change(&w);
        let expected = PrecipitationChange::Stop(Local.timestamp(w.hourly[2].dt, 0), Main::Drizzle);

        assert_eq!(maybe_next_change, expected);
    }

    #[test]
    fn test_next_drizzle_start() {
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
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Drizzle,
            description: "Light Drizzle".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_precipitation_change(&w);
        let expected =
            PrecipitationChange::Start(Local.timestamp(w.hourly[1].dt, 0), Main::Drizzle);

        assert_eq!(maybe_next_change, expected)
    }

    #[test]
    fn test_next_thunderstorm_stop() {
        let mut w: OpenWeather = Default::default();
        w.current.weather = vec![Weather {
            id: 1234,
            main: Main::Thunderstorm,
            description: "Light Thunderstorm".to_string(),
            icon: "some-icon".to_string(),
        }];

        w.hourly = vec![Default::default(), Default::default(), Default::default()];

        w.hourly[0].dt = (Local::now() - chrono::Duration::minutes(30)).timestamp();
        w.hourly[1].dt = (Local::now() + chrono::Duration::hours(1)).timestamp();
        w.hourly[2].dt = (Local::now() + chrono::Duration::hours(2)).timestamp();

        w.hourly[0].weather = vec![Weather {
            id: 2345,
            main: Main::Thunderstorm,
            description: "Light Thunderstorm".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Thunderstorm,
            description: "Light Thunderstorm".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_precipitation_change(&w);
        let expected =
            PrecipitationChange::Stop(Local.timestamp(w.hourly[2].dt, 0), Main::Thunderstorm);

        assert_eq!(maybe_next_change, expected);
    }

    #[test]
    fn test_next_thunderstorm_start() {
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
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Thunderstorm,
            description: "Light Thunderstorm".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_precipitation_change(&w);
        let expected =
            PrecipitationChange::Start(Local.timestamp(w.hourly[1].dt, 0), Main::Thunderstorm);

        assert_eq!(maybe_next_change, expected)
    }

    #[test]
    fn test_next_precipitation_never_starts() {
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

        let maybe_next_change = next_precipitation_change(&w);

        assert_eq!(maybe_next_change, PrecipitationChange::NoChange(None));
    }

    #[test]
    fn test_next_precipitation_stops_changes_precipitation_type() {
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
            main: Main::Snow,
            description: "Light Snow".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[1].weather = vec![Weather {
            id: 2345,
            main: Main::Drizzle,
            description: "Light Drizzle".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_precipitation_change(&w);
        let expected = PrecipitationChange::Stop(Local.timestamp(w.hourly[2].dt, 0), Main::Rain);

        assert_eq!(maybe_next_change, expected)
    }

    #[test]
    fn test_next_precipitation_starts_changes_precipitation_type() {
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
            main: Main::Drizzle,
            description: "Light Drizzle".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_precipitation_change(&w);
        let expected = PrecipitationChange::Start(Local.timestamp(w.hourly[1].dt, 0), Main::Rain);

        assert_eq!(maybe_next_change, expected)
    }

    #[test]
    fn test_next_precipitation_continues_changes_precipitation_type() {
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
            main: Main::Snow,
            description: "Light Snow".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Drizzle,
            description: "Light Drizzle".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_precipitation_change(&w);
        let expected = PrecipitationChange::NoChange(Some(Main::Rain));

        assert_eq!(maybe_next_change, expected)
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

        let maybe_next_change = next_precipitation_change(&w);

        assert_eq!(
            maybe_next_change,
            PrecipitationChange::NoChange(Some(Main::Rain))
        );
    }

    #[test]
    fn test_next_rain_no_change_in_24_hours() {
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
            main: Main::Clear,
            description: "Clear".to_string(),
            icon: "some-icon".to_string(),
        }];
        w.hourly[2].weather = vec![Weather {
            id: 2345,
            main: Main::Rain,
            description: "Light Rain".to_string(),
            icon: "some-icon".to_string(),
        }];

        let maybe_next_change = next_precipitation_change(&w);

        assert_eq!(maybe_next_change, PrecipitationChange::NoChange(None));
    }
}
