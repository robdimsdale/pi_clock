use log::{debug, info};
use simplelog::{ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use structopt::StructOpt;

const CONSOLE_DISPLAY_TYPE: &'static str = "console";
#[cfg(target_arch = "arm")]
const HD44780_DISPLAY_TYPE: &'static str = "hd44780";
#[cfg(target_arch = "arm")]
const ILI9341_DISPLAY_TYPE: &'static str = "ili9341";
#[cfg(target_arch = "arm")]
const ALPHANUM4_DISPLAY_TYPE: &'static str = "alphanum4";
#[cfg(target_arch = "arm")]
const SEVEN_SEGMENT_4_DISPLAY_TYPE: &'static str = "seven_segment4";

const RANDOM_LIGHT_SENSOR_TYPE: &'static str = "random";
const TIME_LIGHT_SENSOR_TYPE: &'static str = "time";
#[cfg(target_arch = "arm")]
const VEML7700_LIGHT_SENSOR_TYPE: &'static str = "veml7700";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_config = ConfigBuilder::new().set_time_to_local(true).build();
    TermLogger::init(LevelFilter::Warn, log_config, TerminalMode::Mixed)?;
    debug!("logger initialized");

    let args = Cli::from_args();

    let light_sensor_type_str = args.light_sensor_type;
    let light_sensor = match light_sensor_type_str.as_str() {
        RANDOM_LIGHT_SENSOR_TYPE => {
            pi_clock::LightSensorType::Random(pi_clock::RandomLightSensor::new())
        }
        TIME_LIGHT_SENSOR_TYPE => pi_clock::LightSensorType::Time(pi_clock::TimeLightSensor::new()),

        #[cfg(target_arch = "arm")]
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
        .map(|d| -> Result<pi_clock::DisplayType<_>, pi_clock::Error> {
            match d.as_str() {
                CONSOLE_DISPLAY_TYPE => Ok(pi_clock::DisplayType::Console(
                    pi_clock::ConsoleDisplay::new(&light_sensor),
                )),

                #[cfg(target_arch = "arm")]
                HD44780_DISPLAY_TYPE => Ok(pi_clock::DisplayType::HD44780(
                    pi_clock::HD44780Display::new(&light_sensor)?,
                )),

                #[cfg(target_arch = "arm")]
                ILI9341_DISPLAY_TYPE => Ok(pi_clock::DisplayType::ILI9341(
                    pi_clock::ILI9341Display::new(&light_sensor)?,
                )),

                #[cfg(target_arch = "arm")]
                ALPHANUM4_DISPLAY_TYPE => Ok(pi_clock::DisplayType::AlphaNum4(
                    pi_clock::AlphaNum4Display::new(&light_sensor)?,
                )),

                #[cfg(target_arch = "arm")]
                SEVEN_SEGMENT_4_DISPLAY_TYPE => Ok(pi_clock::DisplayType::SevenSegment4(
                    pi_clock::SevenSegment4Display::new(&light_sensor)?,
                )),
                _ => {
                    panic!("Unrecognized display type: {}", d)
                }
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut display = pi_clock::DisplayType::Composite(displays.as_mut_slice());

    info!("Initialization complete");

    pi_clock::run(&args.uri, &mut display)?;

    Ok(())
}
#[derive(StructOpt)]
struct Cli {
    #[structopt(long)]
    uri: String,

    #[structopt(long, default_value=RANDOM_LIGHT_SENSOR_TYPE)]
    light_sensor_type: String,

    #[structopt(long = "display-type", default_value=CONSOLE_DISPLAY_TYPE)]
    display_types: Vec<String>,
}
