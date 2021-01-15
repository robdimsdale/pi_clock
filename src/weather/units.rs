use std::error::Error;
use std::str::FromStr;

const UNITS_IMPERIAL: &'static str = "imperial";
const UNITS_METRIC: &'static str = "metric";
const UNITS_STANDARD: &'static str = "standard";

pub enum TemperatureUnits {
    Imperial,
    Metric,
    Standard,
}

impl ToString for TemperatureUnits {
    fn to_string(&self) -> String {
        match self {
            Self::Imperial => UNITS_IMPERIAL.to_owned(),
            Self::Metric => UNITS_METRIC.to_owned(),
            Self::Standard => UNITS_STANDARD.to_owned(),
        }
    }
}

impl FromStr for TemperatureUnits {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<TemperatureUnits, Self::Err> {
        match s {
            UNITS_IMPERIAL => Ok(TemperatureUnits::Imperial),
            UNITS_METRIC => Ok(TemperatureUnits::Metric),
            UNITS_STANDARD => Ok(TemperatureUnits::Standard),
            _ => panic!("Unrecognized temperature units: {}", s),
        }
    }
}

impl TemperatureUnits {
    pub fn as_char(&self) -> char {
        match self {
            Self::Imperial => 'F',
            Self::Metric => 'C',
            Self::Standard => 'K',
        }
    }
}
