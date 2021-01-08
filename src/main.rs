use std::env;

const DISPLAY_TYPE_VAR: &'static str = "DISPLAY";
const OPEN_WEATHER_API_KEY_VAR: &'static str = "OPEN_WEATHER_API_KEY";
const LAT_VAR: &'static str = "LAT";
const LON_VAR: &'static str = "LON";
const UNITS_VAR: &'static str = "UNITS";

#[cfg(target_arch = "arm")]
const DEFAULT_BRIGHTNESS: f64 = 0.05;

const CONSOLE_DISPLAY_TYPE: &'static str = "console";
#[cfg(target_arch = "arm")]
const HD44780_DISPLAY_TYPE: &'static str = "hd44780";
#[cfg(target_arch = "arm")]
const ILI9341_DISPLAY_TYPE: &'static str = "ili9341";

const DEFAULT_UNITS: pi_clock::Units = pi_clock::Units::Imperial;
const DEFAULT_DISPLAY_TYPE: &'static str = CONSOLE_DISPLAY_TYPE;

fn main() {
    println!("Initializing");

    let open_weather_api_key = env::var(OPEN_WEATHER_API_KEY_VAR).expect(&format!(
        "Must provide {} env var",
        OPEN_WEATHER_API_KEY_VAR
    ));
    let lat = env::var(LAT_VAR).expect(&format!("Must provide {} env var", LAT_VAR));
    let lon = env::var(LON_VAR).expect(&format!("Must provide {} env var", LON_VAR));
    let units_str = env::var(UNITS_VAR).unwrap_or(DEFAULT_UNITS.to_string());
    let units = pi_clock::Units::from_string(&units_str);

    let display_type_str = env::var(DISPLAY_TYPE_VAR).unwrap_or(DEFAULT_DISPLAY_TYPE.to_owned());

    let mut display = match display_type_str.as_str() {
        CONSOLE_DISPLAY_TYPE => pi_clock::DisplayType::Console(pi_clock::ConsoleDisplay::new()),

        #[cfg(target_arch = "arm")]
        HD44780_DISPLAY_TYPE => {
            let args: Vec<String> = env::args().collect();

            let brightness = match args.len() {
                0..=1 => DEFAULT_BRIGHTNESS,
                _ => (&args[1]).parse().unwrap_or(DEFAULT_BRIGHTNESS),
            };

            pi_clock::DisplayType::HD44780(pi_clock::HD44780Display::new(brightness))
        }

        #[cfg(target_arch = "arm")]
        ILI9341_DISPLAY_TYPE => {
            let args: Vec<String> = env::args().collect();

            let brightness = match args.len() {
                0..=1 => DEFAULT_BRIGHTNESS,
                _ => (&args[1]).parse().unwrap_or(DEFAULT_BRIGHTNESS),
            };
            pi_clock::DisplayType::ILI9341(pi_clock::ILI9341Display::new(brightness))
        }
        _ => {
            panic!("Unrecognized display type: {}", display_type_str)
        }
    };

    println!("Initialization complete");

    pi_clock::run(&open_weather_api_key, &lat, &lon, &units, &mut display);
}
