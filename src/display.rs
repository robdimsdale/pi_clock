mod error;

use crate::weather::{
    high_low_temp, next_precipitation_change, Main, OpenWeather, PrecipitationChange,
};
pub use error::Error;

use chrono::{DateTime, Datelike, Local, Month, Timelike};
use num_traits::cast::FromPrimitive;

#[cfg(feature = "rpi-hw")]
use hd44780_driver::{
    bus::FourBitBus, Cursor, CursorBlink, Display as HD44780DisplaySetting, DisplayMode, HD44780,
};
#[cfg(feature = "rpi-hw")]
use ht16k33::HT16K33;
#[cfg(feature = "rpi-hw")]
use linux_embedded_hal::sysfs_gpio::Direction;
#[cfg(feature = "rpi-hw")]
use linux_embedded_hal::{Delay, Pin};
#[cfg(feature = "rpi-hw")]
use log::debug;
#[cfg(feature = "rpi-hw")]
use rppal::i2c::I2c;
#[cfg(feature = "rpi-hw")]
use rppal::pwm::{Channel, Polarity, Pwm};

const UNIT_CHAR: char = 'F';

// To enable heterogenous abstractions over multiple display types
pub enum DisplayType<'a> {
    Console16x2(Console16x2Display),
    Console20x4(Console20x4Display),

    #[cfg(feature = "rpi-hw")]
    LCD16x2(LCD16x2Display),
    #[cfg(feature = "rpi-hw")]
    LCD20x4(LCD20x4Display),

    #[cfg(feature = "rpi-hw")]
    AlphaNum4(AlphaNum4Display),

    #[cfg(feature = "rpi-hw")]
    SevenSegment4(SevenSegment4Display),

    Composite(&'a mut [DisplayType<'a>]),
}

impl DisplayType<'_> {
    pub fn print(
        &mut self,
        time: &DateTime<Local>,
        current_state_index: u32,
        weather: &Option<OpenWeather>,
        light: f32,
    ) -> Result<(), Error> {
        match &mut *self {
            Self::Console16x2(display) => display.print(time, current_state_index, weather, light),
            Self::Console20x4(display) => display.print(time, current_state_index, weather, light),

            #[cfg(feature = "rpi-hw")]
            Self::LCD16x2(display) => display.print(time, current_state_index, weather, light),
            #[cfg(feature = "rpi-hw")]
            Self::LCD20x4(display) => display.print(time, current_state_index, weather, light),

            #[cfg(feature = "rpi-hw")]
            Self::AlphaNum4(display) => display.print(time, current_state_index, weather, light),

            #[cfg(feature = "rpi-hw")]
            Self::SevenSegment4(display) => {
                display.print(time, current_state_index, weather, light)
            }

            Self::Composite(displays) => {
                for d in displays.iter_mut() {
                    d.print(time, current_state_index, weather, light)?;
                }
                Ok(())
            }
        }
    }
}

pub trait Display {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        current_state_index: u32,
        weather: &Option<OpenWeather>,
        light: f32,
    ) -> Result<(), Error>;
}

pub struct Console16x2Display {}

impl Console16x2Display {
    pub fn new() -> Console16x2Display {
        Console16x2Display {}
    }
}

impl Default for Console16x2Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Console16x2Display {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        _: u32,
        weather: &Option<OpenWeather>,
        light: f32,
    ) -> Result<(), Error> {
        let (weather_desc, temp_str) = console_weather_and_temp_str(weather, 3, 7);

        let first_row = format!("{} {:>10}", console_time_str(time), weather_desc);
        let second_row = format!("{} {}", console_date_str(time), temp_str);

        println!();
        println!("-{}-", "-".repeat(16));
        println!("|{}|", first_row);
        println!("|{}|", second_row);
        println!("-{}-", "-".repeat(16));

        println!("Current light: {}", light);

        Ok(())
    }
}

fn console_date_str(time: &DateTime<Local>) -> String {
    format!(
        "{} {} {:<2}",
        &time.weekday().to_string()[0..3],
        &mmm_from_time(time),
        time.day()
    )
}

fn console_time_str(time: &DateTime<Local>) -> String {
    let st = split_time(time);
    format!("{}{}:{}{}", st[0], st[1], st[2], st[3])
}

fn console_weather_and_temp_str(
    weather: &Option<OpenWeather>,
    temp_digits: usize,
    weather_chars: usize,
) -> (String, String) {
    match weather {
        Some(w) => (
            format!(
                "{:>width$}",
                truncate_to_characters(&w.current.weather[0].main.to_string(), weather_chars),
                width = weather_chars
            ),
            format!(
                "{:>width$}°{}",
                w.current.temp.round(),
                UNIT_CHAR,
                width = temp_digits
            ),
        ),
        None => (
            format!("{:>width$}", "WEATHER", width = weather_chars),
            format!("{:>width$}", "ERR", width = temp_digits + 2),
        ),
    }
}

pub struct Console20x4Display {}

impl Console20x4Display {
    pub fn new() -> Console20x4Display {
        Console20x4Display {}
    }
}

impl Default for Console20x4Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Console20x4Display {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        current_state_index: u32,
        weather: &Option<OpenWeather>,
        light: f32,
    ) -> Result<(), Error> {
        let (weather_desc, temp_str) = console_weather_and_temp_str(weather, 3, 14);

        let (high_temp_str, low_temp_str) = high_low_strs(weather);

        // time is always 5 chars, date is always 10 chars
        let first_row = format!("{} {:>14}", console_time_str(time), weather_desc);
        let second_row = format!("{} {:>9}", console_date_str(time), temp_str);

        let third_row = format!("{:<20}", "");

        let fourth_row = match current_state_index {
            0 => format!("{:<20}", rain_forecast_str(weather)),
            1 => format!("{:<20}", high_temp_str,),
            2 => format!("{:<20}", low_temp_str),
            _ => panic!("Invalid state index"),
        };

        println!();
        println!("-{}-", "-".repeat(20));
        println!("|{}|", first_row);
        println!("|{}|", second_row);
        println!("|{}|", third_row);
        println!("|{}|", fourth_row);
        println!("-{}-", "-".repeat(20));

        println!("Current light: {}", light);

        Ok(())
    }
}

fn rain_forecast_str(weather: &Option<OpenWeather>) -> String {
    match weather {
        Some(w) => match next_precipitation_change(w) {
            PrecipitationChange::Start(ts, p) => {
                format!("{} starts at {:02}:00", printable_rain_type(p), ts.hour())
            }
            PrecipitationChange::Stop(ts, p) => {
                format!("{} stops at {:02}:00", printable_rain_type(p), ts.hour())
            }
            PrecipitationChange::NoChange(maybe_p) => match maybe_p {
                Some(p) => {
                    format!("{} for next 24h", printable_rain_type(p))
                }
                None => "No rain for next 24h".to_string(),
            },
        },
        None => "".to_string(),
    }
}

fn printable_rain_type(p: Main) -> Main {
    match p {
        Main::Drizzle | Main::Thunderstorm => Main::Rain,
        x => x,
    }
}

fn high_low_strs(weather: &Option<OpenWeather>) -> (String, String) {
    match weather {
        Some(w) => {
            let ((high_time, high_temp), (low_time, low_temp)) = high_low_temp(w);
            (
                format!(
                    "High: {}°F at {:02}:00",
                    high_temp.round(),
                    high_time.hour()
                ),
                format!("Low: {}°F at {:02}:00", low_temp.round(), low_time.hour()),
            )
        }
        None => ("".to_string(), "".to_string()),
    }
}

fn mmm_from_time(time: &DateTime<Local>) -> String {
    Month::from_u32(time.month())
        .expect("failed to parse month from datetime provided by operating system")
        .name()[0..3]
        .to_owned()
}

#[cfg(feature = "rpi-hw")]
pub struct LCD16x2Display {
    lcd: HD44780<
        FourBitBus<
            linux_embedded_hal::Pin,
            linux_embedded_hal::Pin,
            linux_embedded_hal::Pin,
            linux_embedded_hal::Pin,
            linux_embedded_hal::Pin,
            linux_embedded_hal::Pin,
        >,
    >,

    brightness_pwm: Pwm,
}

#[cfg(feature = "rpi-hw")]
impl LCD16x2Display {
    pub fn new() -> Result<Self, Error> {
        // Using BCM numbers
        // i.e. pin 0 corresponds to wiringpi 30 and physical 27

        let rs = Pin::new(21);
        let en = Pin::new(20);
        let db4 = Pin::new(26);
        let db5 = Pin::new(13);
        let db6 = Pin::new(6);
        let db7 = Pin::new(5);
        let r = Pin::new(17);
        let g = Pin::new(16);
        let b = Pin::new(19);

        let default_brightness = 1.0;
        // pwm0 is pin 18
        let pwm0 = Pwm::with_frequency(
            Channel::Pwm0,
            20000.0,
            default_brightness,
            Polarity::Normal,
            false,
        )?;

        pwm0.enable()?;

        rs.export()?;
        en.export()?;
        db4.export()?;
        db5.export()?;
        db6.export()?;
        db7.export()?;
        r.export()?;
        g.export()?;
        b.export()?;

        rs.set_direction(Direction::Low)?;
        en.set_direction(Direction::Low)?;
        db4.set_direction(Direction::Low)?;
        db5.set_direction(Direction::Low)?;
        db6.set_direction(Direction::Low)?;
        db7.set_direction(Direction::Low)?;
        r.set_direction(Direction::Low)?; // Default to red on; green and blue off
        g.set_direction(Direction::High)?;
        b.set_direction(Direction::High)?;

        let mut lcd = HD44780::new_4bit(rs, en, db4, db5, db6, db7, &mut Delay)?;

        lcd.reset(&mut Delay)?;
        lcd.clear(&mut Delay)?;

        lcd.set_display_mode(
            DisplayMode {
                display: HD44780DisplaySetting::On,
                cursor_visibility: Cursor::Invisible,
                cursor_blink: CursorBlink::Off,
            },
            &mut Delay,
        )?;

        Ok(LCD16x2Display {
            lcd,
            brightness_pwm: pwm0,
        })
    }

    fn set_brightness(&mut self, brightness: f32) -> Result<(), Error> {
        debug!("Brightness: {}", brightness);

        self.brightness_pwm.set_duty_cycle(brightness as f64)?;

        Ok(())
    }
}

#[cfg(feature = "rpi-hw")]
impl Display for LCD16x2Display {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        _: u32,
        weather: &Option<OpenWeather>,
        light: f32,
    ) -> Result<(), Error> {
        let (weather_desc, temp_str) = console_weather_and_temp_str(weather, 3, 14);

        // time is always 5 chars, date is always 10 chars
        let first_row = format!("{} {:>14}", console_time_str(time), weather_desc);
        let second_row = format!("{} {:>9}", console_date_str(time), temp_str);

        // Move to beginning of first row.
        self.lcd.reset(&mut Delay)?;

        self.lcd
            .write_bytes(&str_to_lcd_bytes(&first_row), &mut Delay)?;

        // Move to line 2
        self.lcd.set_cursor_pos(0x40, &mut Delay)?;

        self.lcd
            .write_bytes(&str_to_lcd_bytes(&second_row), &mut Delay)?;

        let min_brightness = 0.01;
        let light = light.max(min_brightness);

        self.set_brightness(light)?;

        Ok(())
    }
}

#[cfg(feature = "rpi-hw")]
pub struct LCD20x4Display {
    lcd: HD44780<
        FourBitBus<
            linux_embedded_hal::Pin,
            linux_embedded_hal::Pin,
            linux_embedded_hal::Pin,
            linux_embedded_hal::Pin,
            linux_embedded_hal::Pin,
            linux_embedded_hal::Pin,
        >,
    >,

    brightness_pwm: Pwm,
}

#[cfg(feature = "rpi-hw")]
impl LCD20x4Display {
    pub fn new() -> Result<Self, Error> {
        // Using BCM numbers
        // i.e. pin 0 corresponds to wiringpi 30 and physical 27

        let rs = Pin::new(21);
        let en = Pin::new(20);
        let db4 = Pin::new(19); // prev: 26
        let db5 = Pin::new(13);
        let db6 = Pin::new(6);
        let db7 = Pin::new(5);
        let r = Pin::new(17);
        // let g = Pin::new(16);
        // let b = Pin::new(19);

        let default_brightness = 1.0;
        // pwm0 is pin 18
        let pwm0 = Pwm::with_frequency(
            Channel::Pwm0,
            20000.0,
            default_brightness,
            Polarity::Normal,
            false,
        )?;

        pwm0.enable()?;

        rs.export()?;
        en.export()?;
        db4.export()?;
        db5.export()?;
        db6.export()?;
        db7.export()?;
        r.export()?;
        // g.export()?;
        // b.export()?;

        rs.set_direction(Direction::Low)?;
        en.set_direction(Direction::Low)?;
        db4.set_direction(Direction::Low)?;
        db5.set_direction(Direction::Low)?;
        db6.set_direction(Direction::Low)?;
        db7.set_direction(Direction::Low)?;
        r.set_direction(Direction::Low)?; // Default to red on; green and blue off
                                          // g.set_direction(Direction::High)?;
                                          // b.set_direction(Direction::High)?;

        let mut lcd = HD44780::new_4bit(rs, en, db4, db5, db6, db7, &mut Delay)?;

        lcd.reset(&mut Delay)?;
        lcd.clear(&mut Delay)?;

        lcd.set_display_mode(
            DisplayMode {
                display: HD44780DisplaySetting::On,
                cursor_visibility: Cursor::Invisible,
                cursor_blink: CursorBlink::Off,
            },
            &mut Delay,
        )?;

        Ok(LCD20x4Display {
            lcd,
            brightness_pwm: pwm0,
        })
    }

    fn set_brightness(&mut self, brightness: f32) -> Result<(), Error> {
        debug!("Brightness: {}", brightness);

        self.brightness_pwm.set_duty_cycle(brightness as f64)?;

        Ok(())
    }
}

#[cfg(feature = "rpi-hw")]
impl Display for LCD20x4Display {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        current_state_index: u32,
        weather: &Option<OpenWeather>,
        light: f32,
    ) -> Result<(), Error> {
        let (weather_desc, temp_str) = console_weather_and_temp_str(weather, 3, 14);
        let (high_temp_str, low_temp_str) = high_low_strs(weather);

        // time is always 5 chars, date is always 10 chars
        let first_row = format!("{} {:>14}", console_time_str(time), weather_desc);
        let second_row = format!("{} {:>9}", console_date_str(time), temp_str);
        let third_row = "";

        let fourth_row = match current_state_index {
            0 => format!("{:<20}", rain_forecast_str(weather)),
            1 => format!("{:<20}", high_temp_str),
            2 => format!("{:<20}", low_temp_str),
            _ => panic!("Invalid state index"),
        };

        // Move to beginning of first row.
        self.lcd.reset(&mut Delay)?;

        self.lcd
            .write_bytes(&str_to_lcd_bytes(&first_row), &mut Delay)?;

        // Move to line 2
        self.lcd.set_cursor_pos(0x40, &mut Delay)?;

        self.lcd
            .write_bytes(&str_to_lcd_bytes(&second_row), &mut Delay)?;

        // Move to line 3
        self.lcd.set_cursor_pos(0x14, &mut Delay)?;

        self.lcd
            .write_bytes(&str_to_lcd_bytes(third_row), &mut Delay)?;

        // Move to line 4
        self.lcd.set_cursor_pos(0x54, &mut Delay)?;

        self.lcd
            .write_bytes(&str_to_lcd_bytes(&fourth_row), &mut Delay)?;

        let min_brightness = 0.01;
        let light = light.max(min_brightness);

        self.set_brightness(light)?;

        Ok(())
    }
}

#[cfg(feature = "rpi-hw")]
fn str_to_lcd_bytes(s: &str) -> Vec<u8> {
    s.replace('°', "#") // Pick a character that we know won't appear in the string elsewhere
        .as_bytes()
        .iter()
        .map(|&i| if i == b'#' { 0xDF } else { i }) // 0xDF is the bytecode for the ° symbol
        .collect::<Vec<u8>>()
}

#[cfg(feature = "rpi-hw")]
pub struct AlphaNum4Display {
    ht16k33: HT16K33<I2c>,
}

#[cfg(feature = "rpi-hw")]
impl AlphaNum4Display {
    pub fn new() -> Result<Self, Error> {
        // The I2C device address.
        let address = 0x71;

        // Create an I2C device.
        let mut i2c = I2c::new()?;
        i2c.set_slave_address(address as u16)?;

        let mut ht16k33 = HT16K33::new(i2c, address);
        ht16k33.initialize()?;

        ht16k33.set_display(ht16k33::Display::ON)?;

        Ok(AlphaNum4Display { ht16k33 })
    }

    fn set_brightness(&mut self, light: f32) -> Result<(), Error> {
        let level = (light * 15.0).round() as u8;
        let dimming = ht16k33::Dimming::from_u8(level)?;

        debug!(
            "Current light level: {}, dimming level: {}/16",
            light, level
        );

        self.ht16k33.set_dimming(dimming)?;

        Ok(())
    }
}

#[cfg(feature = "rpi-hw")]
impl Display for AlphaNum4Display {
    fn print(
        &mut self,
        _: &DateTime<Local>,
        _: u32,
        weather: &Option<OpenWeather>,
        light: f32,
    ) -> Result<(), Error> {
        let [d1, d2, d3] = match weather {
            Some(w) => {
                let chars = format!("{:>3}", w.current.temp.round())
                    .chars()
                    .collect::<Vec<char>>();
                [chars[0], chars[1], chars[2]]
            }
            None => ['E', 'R', 'R'],
        };

        let d4 = match weather {
            Some(_) => UNIT_CHAR,
            None => ' ',
        };
        adafruit_alphanum4::AlphaNum4::update_buffer_with_char(
            &mut self.ht16k33,
            adafruit_alphanum4::Index::One,
            adafruit_alphanum4::AsciiChar::new(d1),
        );
        adafruit_alphanum4::AlphaNum4::update_buffer_with_char(
            &mut self.ht16k33,
            adafruit_alphanum4::Index::Two,
            adafruit_alphanum4::AsciiChar::new(d2),
        );
        adafruit_alphanum4::AlphaNum4::update_buffer_with_char(
            &mut self.ht16k33,
            adafruit_alphanum4::Index::Three,
            adafruit_alphanum4::AsciiChar::new(d3),
        );
        adafruit_alphanum4::AlphaNum4::update_buffer_with_char(
            &mut self.ht16k33,
            adafruit_alphanum4::Index::Four,
            adafruit_alphanum4::AsciiChar::new(d4),
        );

        self.ht16k33.write_display_buffer()?;

        self.set_brightness(light)?;

        Ok(())
    }
}

#[cfg(feature = "rpi-hw")]
pub struct SevenSegment4Display {
    ht16k33: HT16K33<I2c>,
}

#[cfg(feature = "rpi-hw")]
impl SevenSegment4Display {
    pub fn new() -> Result<Self, Error> {
        // The I2C device address.
        let address = 0x70;

        // Create an I2C device.
        let mut i2c = I2c::new()?;
        i2c.set_slave_address(address as u16)?;

        let mut ht16k33 = HT16K33::new(i2c, address);
        ht16k33.initialize()?;

        ht16k33.set_display(ht16k33::Display::ON)?;

        Ok(SevenSegment4Display { ht16k33 })
    }

    fn set_brightness(&mut self, brightness: f32) -> Result<(), Error> {
        let level = (brightness * 15.0).round() as u8;
        let dimming = ht16k33::Dimming::from_u8(level)?;

        debug!("Brightness: {}, dimming level: {}/16", brightness, level);

        self.ht16k33.set_dimming(dimming)?;

        Ok(())
    }
}

#[cfg(feature = "rpi-hw")]
impl Display for SevenSegment4Display {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        _: u32,
        _: &Option<OpenWeather>,
        light: f32,
    ) -> Result<(), Error> {
        let [d1, d2, d3, d4] = split_time(time);
        adafruit_7segment::SevenSegment::update_buffer_with_digit(
            &mut self.ht16k33,
            adafruit_7segment::Index::One,
            d1,
        );
        adafruit_7segment::SevenSegment::update_buffer_with_digit(
            &mut self.ht16k33,
            adafruit_7segment::Index::Two,
            d2,
        );
        adafruit_7segment::SevenSegment::update_buffer_with_digit(
            &mut self.ht16k33,
            adafruit_7segment::Index::Three,
            d3,
        );
        adafruit_7segment::SevenSegment::update_buffer_with_digit(
            &mut self.ht16k33,
            adafruit_7segment::Index::Four,
            d4,
        );
        adafruit_7segment::SevenSegment::update_buffer_with_colon(&mut self.ht16k33, true);
        self.ht16k33.write_display_buffer()?;

        self.set_brightness(light)?;

        Ok(())
    }
}

fn split_time(t: &DateTime<Local>) -> [u8; 4] {
    let hour = t.hour();
    let minute = t.minute();

    let d4 = (minute % 10) as u8;
    let d3 = (minute / 10) as u8 % 10;

    let d2 = (hour % 10) as u8;
    let d1 = (hour / 10) as u8 % 10;

    [d1, d2, d3, d4]
}

fn truncate_to_characters(s: &str, length: usize) -> String {
    if s.len() <= length {
        return s.to_owned();
    }

    format!("{}'{}", &s[0..1], &s[s.len() - length + 2..s.len()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_to_characters() {
        assert_eq!(truncate_to_characters("", 3), "");
        assert_eq!(truncate_to_characters("a", 3), "a");
        assert_eq!(truncate_to_characters("ab", 3), "ab");
        assert_eq!(truncate_to_characters("abc", 3), "abc");
        assert_eq!(truncate_to_characters("abcd", 3), "a'd");
        assert_eq!(truncate_to_characters("abcdefg", 5), "a'efg");
        assert_eq!(truncate_to_characters("Tornado", 7), "Tornado");
        assert_eq!(truncate_to_characters("Thunderstorm", 7), "T'storm");
    }

    #[test]
    fn test_split_time() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            split_time(&Local::now().with_hour(1).unwrap().with_minute(3).unwrap()),
            [0, 1, 0, 3]
        );
        assert_eq!(
            split_time(&Local::now().with_hour(0).unwrap().with_minute(0).unwrap()),
            [0, 0, 0, 0]
        );
        assert_eq!(
            split_time(&Local::now().with_hour(12).unwrap().with_minute(34).unwrap()),
            [1, 2, 3, 4]
        );
        assert_eq!(
            split_time(&Local::now().with_hour(23).unwrap().with_minute(59).unwrap()),
            [2, 3, 5, 9]
        );

        Ok(())
    }
}
