use log::{debug, info};
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use std::time::Duration;
use structopt::StructOpt;

const CONSOLE_16X2_DISPLAY_TYPE: &str = "console-16x2";
const CONSOLE_20X4_DISPLAY_TYPE: &str = "console-20x4";

#[cfg(feature = "rpi-hw")]
const LCD_16X2_DISPLAY_TYPE: &str = "lcd-16x2";
#[cfg(feature = "rpi-hw")]
const LCD_20X4_DISPLAY_TYPE: &str = "lcd-20x4";
#[cfg(feature = "rpi-hw")]
const ALPHANUM4_DISPLAY_TYPE: &str = "alphanum4";
#[cfg(feature = "rpi-hw")]
const SEVEN_SEGMENT_4_DISPLAY_TYPE: &str = "seven_segment4";

const RANDOM_LIGHT_SENSOR_TYPE: &str = "random";
const TIME_LIGHT_SENSOR_TYPE: &str = "time";
#[cfg(feature = "rpi-hw")]
const VEML7700_LIGHT_SENSOR_TYPE: &str = "veml7700";

#[cfg(not(feature = "rpi-hw"))]
const VALID_DISPLAY_TYPES: &[&str] = &[CONSOLE_16X2_DISPLAY_TYPE, CONSOLE_20X4_DISPLAY_TYPE];

#[cfg(feature = "rpi-hw")]
const VALID_DISPLAY_TYPES: &[&str] = &[
    CONSOLE_16X2_DISPLAY_TYPE,
    CONSOLE_20X4_DISPLAY_TYPE,
    LCD_16X2_DISPLAY_TYPE,
    LCD_20X4_DISPLAY_TYPE,
    ALPHANUM4_DISPLAY_TYPE,
    SEVEN_SEGMENT_4_DISPLAY_TYPE,
];

#[cfg(not(feature = "rpi-hw"))]
const VALID_LIGHT_SENSOR_TYPES: &[&str] = &[RANDOM_LIGHT_SENSOR_TYPE, TIME_LIGHT_SENSOR_TYPE];

#[cfg(feature = "rpi-hw")]
const VALID_LIGHT_SENSOR_TYPES: &[&str] = &[
    RANDOM_LIGHT_SENSOR_TYPE,
    TIME_LIGHT_SENSOR_TYPE,
    VEML7700_LIGHT_SENSOR_TYPE,
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_config = ConfigBuilder::new()
        .set_time_offset_to_local()
        .unwrap_or_else(|e| e)
        .set_time_format_rfc2822()
        // .set_time_format_str("%F %T") // i.e. '2020-02-27 15:12:02'
        .build();
    TermLogger::init(
        LevelFilter::Warn,
        log_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;
    debug!("logger initialized");

    let args = Cli::from_args();

    let light_sensor_type_str = args.light_sensor_type;
    let light_sensor = match light_sensor_type_str.as_str() {
        RANDOM_LIGHT_SENSOR_TYPE => {
            pi_clock::LightSensorType::Random(pi_clock::RandomLightSensor::new())
        }
        TIME_LIGHT_SENSOR_TYPE => pi_clock::LightSensorType::Time(pi_clock::TimeLightSensor::new()),

        #[cfg(feature = "rpi-hw")]
        VEML7700_LIGHT_SENSOR_TYPE => {
            pi_clock::LightSensorType::VEML7700(pi_clock::VEML7700LightSensor::new()?)
        }
        _ => {
            panic!("Unrecognized light sensor type: {}", light_sensor_type_str)
        }
    };

    let mut displays = args
        .display_types
        .iter()
        .map(|d| -> Result<pi_clock::DisplayType, pi_clock::Error> {
            match d.as_str() {
                CONSOLE_16X2_DISPLAY_TYPE => Ok(pi_clock::DisplayType::Console16x2(
                    pi_clock::Console16x2Display::new(),
                )),

                CONSOLE_20X4_DISPLAY_TYPE => Ok(pi_clock::DisplayType::Console20x4(
                    pi_clock::Console20x4Display::new(),
                )),

                #[cfg(feature = "rpi-hw")]
                LCD_16X2_DISPLAY_TYPE => Ok(pi_clock::DisplayType::LCD16x2(
                    pi_clock::LCD16x2Display::new()?,
                )),

                #[cfg(feature = "rpi-hw")]
                LCD_20X4_DISPLAY_TYPE => Ok(pi_clock::DisplayType::LCD20x4(
                    pi_clock::LCD20x4Display::new()?,
                )),

                #[cfg(feature = "rpi-hw")]
                ALPHANUM4_DISPLAY_TYPE => Ok(pi_clock::DisplayType::AlphaNum4(
                    pi_clock::AlphaNum4Display::new()?,
                )),

                #[cfg(feature = "rpi-hw")]
                SEVEN_SEGMENT_4_DISPLAY_TYPE => Ok(pi_clock::DisplayType::SevenSegment4(
                    pi_clock::SevenSegment4Display::new()?,
                )),
                _ => {
                    panic!("Unrecognized display type: {}", d)
                }
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut display = pi_clock::DisplayType::Composite(displays.as_mut_slice());

    info!("Initialization complete");

    let config = pi_clock::Config {
        uri: args.uri,
        loop_sleep_duration: Duration::from_millis(args.loop_duration_millis),
        state_duration: Duration::from_secs(args.state_duration_secs),
        weather_request_timeout: Duration::from_millis(args.weather_request_timeout_millis),
        weather_request_polling_interval: Duration::from_secs(
            args.weather_request_polling_interval_secs,
        ),
    };

    pi_clock::run(&config, &mut display, &light_sensor)?;

    Ok(())
}
#[derive(StructOpt)]
struct Cli {
    #[structopt(long)]
    uri: String,

    #[structopt(long, default_value = "500")]
    loop_duration_millis: u64,

    #[structopt(long, default_value = "5")]
    weather_request_polling_interval_secs: u64,

    #[structopt(long, default_value = "200")]
    weather_request_timeout_millis: u64,

    #[structopt(long, default_value = "3")]
    state_duration_secs: u64,

    #[structopt(long, possible_values(VALID_LIGHT_SENSOR_TYPES),default_value=RANDOM_LIGHT_SENSOR_TYPE)]
    light_sensor_type: String,

    #[structopt(long = "display-type", possible_values(VALID_DISPLAY_TYPES),default_value=CONSOLE_16X2_DISPLAY_TYPE)]
    display_types: Vec<String>,
}
