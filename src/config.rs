use std::time::Duration;

pub struct Config {
    pub loop_sleep_duration: Duration,
    pub uri: String,
    pub weather_request_polling_interval: Duration,
    pub weather_request_timeout: Duration,
    pub state_duration: Duration,
}
