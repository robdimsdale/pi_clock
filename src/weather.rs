mod open_weather_types;
mod units;

pub use open_weather_types::OpenWeather;
use std::fmt;
pub use units::TemperatureUnits;

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
    HTTP(ureq::Error),
    StringParse(std::io::Error),
    JSONParse(serde_json::Error),
    Transport(ureq::Error),
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
            ErrorKind::HTTP(ref err) => err.fmt(f),
            ErrorKind::StringParse(ref err) => err.fmt(f),
            ErrorKind::JSONParse(ref err) => err.fmt(f),
            ErrorKind::Transport(ref err) => err.fmt(f),
            ErrorKind::__Nonexhaustive => unreachable!(),
        }
    }
}

impl From<ureq::Error> for Error {
    fn from(e: ureq::Error) -> Self {
        match e {
            ureq::Error::Status(_, _) => Error {
                kind: ErrorKind::HTTP(e),
            },
            ureq::Error::Transport(_) => Error {
                kind: ErrorKind::Transport(e),
            },
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error {
            kind: ErrorKind::StringParse(e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error {
            kind: ErrorKind::JSONParse(e),
        }
    }
}

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

    let mut req = ureq::get(&uri); // TODO: add timeout.

    let response = req.call()?;

    Ok(serde_json::from_str(&response.into_string()?)?)
}
