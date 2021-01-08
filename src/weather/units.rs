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

impl TemperatureUnits {
    pub fn from_string(s: &str) -> TemperatureUnits {
        match s {
            UNITS_IMPERIAL => TemperatureUnits::Imperial,
            UNITS_METRIC => TemperatureUnits::Metric,
            UNITS_STANDARD => TemperatureUnits::Standard,
            _ => panic!("Unrecognized temperature units: {}", s),
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            Self::Imperial => 'F',
            Self::Metric => 'C',
            Self::Standard => 'K',
        }
    }
}
