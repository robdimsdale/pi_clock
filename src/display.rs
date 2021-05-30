mod error;

use crate::light::LightSensor;
use crate::weather::{currently_raining, high_low_temp, next_rain_start_or_stop, OpenWeather};
pub use error::Error;

use chrono::{DateTime, Datelike, Local, Month, Timelike};
use num_traits::cast::FromPrimitive;

#[cfg(target_arch = "arm")]
use embedded_graphics::{
    egrectangle, egtext, fonts::Font12x16, fonts::Font24x32, pixelcolor::Rgb565, prelude::*,
    primitive_style, text_style,
};
#[cfg(target_arch = "arm")]
use hd44780_driver::{
    bus::FourBitBus, Cursor, CursorBlink, Display as HD44780DisplaySetting, DisplayMode, HD44780,
};
#[cfg(target_arch = "arm")]
use ht16k33::HT16K33;
#[cfg(target_arch = "arm")]
use ili9341::{Ili9341, Orientation};
#[cfg(target_arch = "arm")]
use linux_embedded_hal::sysfs_gpio::Direction;
#[cfg(target_arch = "arm")]
use linux_embedded_hal::{Delay, Pin};
#[cfg(target_arch = "arm")]
use log::debug;
#[cfg(target_arch = "arm")]
use rppal::i2c::I2c;
#[cfg(target_arch = "arm")]
use rppal::pwm::{Channel, Polarity, Pwm};
#[cfg(target_arch = "arm")]
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

const UNIT_CHAR: char = 'F';

// To enable heterogenous abstractions over multiple display types
pub enum DisplayType<'a, T: LightSensor> {
    Console16x2(Console16x2Display<'a, T>),
    Console20x4(Console20x4Display<'a, T>),

    #[cfg(target_arch = "arm")]
    LCD16x2(LCD16x2Display<'a, T>),
    #[cfg(target_arch = "arm")]
    LCD20x4(LCD20x4Display<'a, T>),

    #[cfg(target_arch = "arm")]
    ILI9341(ILI9341Display<'a, T>),

    #[cfg(target_arch = "arm")]
    AlphaNum4(AlphaNum4Display<'a, T>),

    #[cfg(target_arch = "arm")]
    SevenSegment4(SevenSegment4Display<'a, T>),

    Composite(&'a mut [DisplayType<'a, T>]),
}

impl<'a, T: LightSensor> DisplayType<'a, T> {
    pub fn print(
        &mut self,
        time: &DateTime<Local>,
        current_state_index: u32,
        weather: &Option<OpenWeather>,
    ) -> Result<(), Error> {
        match &mut *self {
            Self::Console16x2(display) => display.print(time, current_state_index, weather),
            Self::Console20x4(display) => display.print(time, current_state_index, weather),

            #[cfg(target_arch = "arm")]
            Self::LCD16x2(display) => display.print(time, current_state_index, weather),
            #[cfg(target_arch = "arm")]
            Self::LCD20x4(display) => display.print(time, current_state_index, weather),

            #[cfg(target_arch = "arm")]
            Self::ILI9341(display) => display.print(time, current_state_index, weather),

            #[cfg(target_arch = "arm")]
            Self::AlphaNum4(display) => display.print(time, current_state_index, weather),

            #[cfg(target_arch = "arm")]
            Self::SevenSegment4(display) => display.print(time, current_state_index, weather),

            Self::Composite(displays) => {
                for d in displays.iter_mut() {
                    d.print(time, current_state_index, weather)?;
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
    ) -> Result<(), Error>;
}

pub struct Console16x2Display<'a, T: LightSensor> {
    light_sensor: &'a T,
}

impl<'a, T: LightSensor> Console16x2Display<'a, T> {
    pub fn new(light_sensor: &'a T) -> Console16x2Display<'a, T> {
        Console16x2Display {
            light_sensor: light_sensor,
        }
    }
}

impl<'a, T: LightSensor> Display for Console16x2Display<'a, T> {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        _: u32,
        weather: &Option<OpenWeather>,
    ) -> Result<(), Error> {
        let (weather_desc, temp_str) = console_weather_and_temp_str(&weather, 3, 7);

        let first_row = format!("{} {:>10}", console_time_str(&time), weather_desc);
        let second_row = format!("{} {}", console_date_str(&time), temp_str);

        println!();
        println!("-{}-", std::iter::repeat("-").take(16).collect::<String>());
        println!("|{}|", first_row);
        println!("|{}|", second_row);
        println!("-{}-", std::iter::repeat("-").take(16).collect::<String>());

        println!(
            "Current light: {}",
            self.light_sensor.read_light_normalized()?
        );

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
    let st = split_time(time).unwrap(); // TODO: remove error from signature
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
                truncate_to_characters(&w.current.weather[0].main, weather_chars),
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

pub struct Console20x4Display<'a, T: LightSensor> {
    light_sensor: &'a T,
}

impl<'a, T: LightSensor> Console20x4Display<'a, T> {
    pub fn new(light_sensor: &'a T) -> Console20x4Display<'a, T> {
        Console20x4Display {
            light_sensor: light_sensor,
        }
    }
}

impl<'a, T: LightSensor> Display for Console20x4Display<'a, T> {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        current_state_index: u32,
        weather: &Option<OpenWeather>,
    ) -> Result<(), Error> {
        let (weather_desc, temp_str) = console_weather_and_temp_str(&weather, 3, 14);

        let (high_temp_str, low_temp_str) = high_low_strs(&weather);

        // time is always 5 chars, date is always 10 chars
        let first_row = format!("{} {:>14}", console_time_str(&time), weather_desc);
        let second_row = format!("{} {:>9}", console_date_str(&time), temp_str);

        let third_row = format!("{:<20}", "");

        let fourth_row = match current_state_index {
            0 => format!("{:<20}", rain_forecast_str(&weather)),
            1 => format!("{:<20}", high_temp_str,),
            2 => format!("{:<20}", low_temp_str),
            _ => panic!("Invalid state index"),
        };

        println!();
        println!("-{}-", std::iter::repeat("-").take(20).collect::<String>());
        println!("|{}|", first_row);
        println!("|{}|", second_row);
        println!("|{}|", third_row);
        println!("|{}|", fourth_row);
        println!("-{}-", std::iter::repeat("-").take(20).collect::<String>());

        println!(
            "Current light: {}",
            self.light_sensor.read_light_normalized()?
        );

        Ok(())
    }
}

fn rain_forecast_str(weather: &Option<OpenWeather>) -> String {
    match weather {
        Some(w) => match next_rain_start_or_stop(&w) {
            Some(ts) => {
                if currently_raining(&w) {
                    format!("Rain stops at {:02}:00", ts.hour())
                } else {
                    format!("Rain starts at {:02}:00", ts.hour())
                }
            }
            None => {
                if currently_raining(&w) {
                    "Rain for next 24h".to_string()
                } else {
                    "No rain for next 24h".to_string()
                }
            }
        },
        None => "".to_string(),
    }
}

fn high_low_strs(weather: &Option<OpenWeather>) -> (String, String) {
    match weather {
        Some(w) => {
            let ((high_time, high_temp), (low_time, low_temp)) = high_low_temp(&w);
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

#[cfg(target_arch = "arm")]
pub struct LCD16x2Display<'a, T: LightSensor> {
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

    light_sensor: &'a T,
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> LCD16x2Display<'a, T> {
    pub fn new(light_sensor: &'a T) -> Result<Self, Error> {
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
            lcd: lcd,
            brightness_pwm: pwm0,
            light_sensor: light_sensor,
        })
    }

    fn set_brightness(&mut self, brightness: f32) -> Result<(), Error> {
        debug!("Brightness: {}", brightness);

        self.brightness_pwm.set_duty_cycle(brightness as f64)?;

        Ok(())
    }
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> Display for LCD16x2Display<'a, T> {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        _: u32,
        weather: &Option<OpenWeather>,
    ) -> Result<(), Error> {
        let (weather_desc, temp_str) = console_weather_and_temp_str(&weather, 3, 14);

        // time is always 5 chars, date is always 10 chars
        let first_row = format!("{} {:>14}", console_time_str(&time), weather_desc);
        let second_row = format!("{} {:>9}", console_date_str(&time), temp_str);

        // Move to beginning of first row.
        self.lcd.reset(&mut Delay)?;

        self.lcd
            .write_bytes(&str_to_lcd_bytes(&first_row), &mut Delay)?;

        // Move to line 2
        self.lcd.set_cursor_pos(0x40, &mut Delay)?;

        self.lcd
            .write_bytes(&str_to_lcd_bytes(&second_row), &mut Delay)?;

        let brightness = self.light_sensor.read_light_normalized()?;
        let min_brightness = 0.01;
        let brightness = brightness.max(min_brightness);

        self.set_brightness(brightness)?;

        Ok(())
    }
}

#[cfg(target_arch = "arm")]
pub struct LCD20x4Display<'a, T: LightSensor> {
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

    light_sensor: &'a T,
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> LCD20x4Display<'a, T> {
    pub fn new(light_sensor: &'a T) -> Result<Self, Error> {
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
            lcd: lcd,
            brightness_pwm: pwm0,
            light_sensor: light_sensor,
        })
    }

    fn set_brightness(&mut self, brightness: f32) -> Result<(), Error> {
        debug!("Brightness: {}", brightness);

        self.brightness_pwm.set_duty_cycle(brightness as f64)?;

        Ok(())
    }
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> Display for LCD20x4Display<'a, T> {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        current_state_index: u32,
        weather: &Option<OpenWeather>,
    ) -> Result<(), Error> {
        let (weather_desc, temp_str) = console_weather_and_temp_str(&weather, 3, 14);
        let (high_temp_str, low_temp_str) = high_low_strs(&weather);

        // time is always 5 chars, date is always 10 chars
        let first_row = format!("{} {:>14}", console_time_str(&time), weather_desc);
        let second_row = format!("{} {:>9}", console_date_str(&time), temp_str);
        let third_row = "";

        let fourth_row = match current_state_index {
            0 => format!("{:<20}", rain_forecast_str(&weather)),
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
            .write_bytes(&str_to_lcd_bytes(&third_row), &mut Delay)?;

        // Move to line 4
        self.lcd.set_cursor_pos(0x54, &mut Delay)?;

        self.lcd
            .write_bytes(&str_to_lcd_bytes(&fourth_row), &mut Delay)?;

        let brightness = self.light_sensor.read_light_normalized()?;
        let min_brightness = 0.01;
        let brightness = brightness.max(min_brightness);

        self.set_brightness(brightness)?;

        Ok(())
    }
}

fn str_to_lcd_bytes(s: &str) -> Vec<u8> {
    s.replace("°", "#") // Pick a character that we know won't appear in the string elsewhere
        .as_bytes()
        .iter()
        .map(|&i| if i == '#' as u8 { 0xDF } else { i }) // 0xDF is the bytecode for the ° symbol
        .collect::<Vec<u8>>()
}

#[cfg(target_arch = "arm")]
pub struct ILI9341Display<'a, T: LightSensor> {
    display: Ili9341<
        display_interface_spi::SPIInterface<Spi, linux_embedded_hal::Pin, linux_embedded_hal::Pin>,
        linux_embedded_hal::Pin,
    >,
    brightness_pwm: Pwm,
    light_sensor: &'a T,
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> ILI9341Display<'a, T> {
    pub fn new(light_sensor: &'a T) -> Result<Self, Error> {
        // Using BCM numbers
        // i.e. pin 0 corresponds to wiringpi 30 and physical 27

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

        let rs = Pin::new(24);
        rs.export()?;
        rs.set_direction(Direction::Low)?;

        let cs = Pin::new(21); // TODO: can't use the CE0 pin in the display as it is already used by the SPI variable.
        cs.export()?;
        cs.set_direction(Direction::Low)?;

        let dc = Pin::new(25);
        dc.export()?;
        dc.set_direction(Direction::Low)?;

        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0)?;

        let spi_di = display_interface_spi::SPIInterface::new(spi, dc, cs);

        let mut display = Ili9341::new(spi_di, rs, &mut Delay)?;

        display.set_orientation(Orientation::LandscapeFlipped)?;

        Ok(ILI9341Display {
            display: display,
            brightness_pwm: pwm0,
            light_sensor: light_sensor,
        })
    }

    fn set_brightness(&mut self, brightness: f32) -> Result<(), Error> {
        debug!("LED brightness: {}", brightness);

        self.brightness_pwm.set_duty_cycle(brightness as f64)?;

        Ok(())
    }
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> Display for ILI9341Display<'a, T> {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        _: u32,
        weather: &Option<OpenWeather>,
    ) -> Result<(), Error> {
        let day = &time.weekday().to_string()[0..3];
        let month = &mmm_from_time(time);

        let first_row = format!("{:02}:{:02}", time.hour(), time.minute());

        let second_row = format!("{} {} {:<2}", day, month, time.day());
        let (third_row, fourth_row) = match weather {
            Some(w) => (
                format!("{}", truncate_to_characters(&w.current.weather[0].main, 7)),
                format!("{:>3}°{}", &w.current.temp.round(), UNIT_CHAR),
            ),
            None => ("WEATHER".to_owned(), "ERR".to_owned()),
        };

        let text = format!("{}\n{}\n{}", second_row, third_row, fourth_row);

        let background = egrectangle!(
            top_left = (0, 0),
            bottom_right = (320, 240),
            style = primitive_style!(fill_color = Rgb565::BLACK),
        );

        let time_text = egtext!(
            text = &first_row,
            top_left = (20, 16),
            style = text_style!(font = Font24x32, text_color = Rgb565::RED),
        );

        let other_text = egtext!(
            text = &text,
            top_left = (20, 48),
            style = text_style!(font = Font12x16, text_color = Rgb565::RED),
        );

        background.draw(&mut self.display)?;
        time_text.draw(&mut self.display)?;
        other_text.draw(&mut self.display)?;

        let brightness = self.light_sensor.read_light_normalized()?;
        let min_brightness = 0.01;
        let brightness = brightness.max(min_brightness);

        self.set_brightness(brightness)?;

        Ok(())
    }
}

#[cfg(target_arch = "arm")]
pub struct AlphaNum4Display<'a, T: LightSensor> {
    ht16k33: HT16K33<I2c>,

    light_sensor: &'a T,
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> AlphaNum4Display<'a, T> {
    pub fn new(light_sensor: &'a T) -> Result<Self, Error> {
        // The I2C device address.
        let address = 0x71;

        // Create an I2C device.
        let mut i2c = I2c::new()?;
        i2c.set_slave_address(address as u16)?;

        let mut ht16k33 = HT16K33::new(i2c, address);
        ht16k33.initialize()?;

        ht16k33.set_display(ht16k33::Display::ON)?;

        Ok(AlphaNum4Display {
            ht16k33: ht16k33,
            light_sensor: light_sensor,
        })
    }

    fn set_brightness(&mut self, brightness: f32) -> Result<(), Error> {
        let level = (brightness * 15.0).round() as u8;
        let dimming = ht16k33::Dimming::from_u8(level)?;

        debug!(
            "Current light level: {}, dimming level: {}/16",
            brightness, level
        );

        self.ht16k33.set_dimming(dimming)?;

        Ok(())
    }
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> Display for AlphaNum4Display<'a, T> {
    fn print(
        &mut self,
        _: &DateTime<Local>,
        _: u32,
        weather: &Option<OpenWeather>,
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

        let brightness = self.light_sensor.read_light_normalized()?;
        self.set_brightness(brightness)?;

        Ok(())
    }
}

#[cfg(target_arch = "arm")]
pub struct SevenSegment4Display<'a, T: LightSensor> {
    ht16k33: HT16K33<I2c>,

    light_sensor: &'a T,
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> SevenSegment4Display<'a, T> {
    pub fn new(light_sensor: &'a T) -> Result<Self, Error> {
        // The I2C device address.
        let address = 0x70;

        // Create an I2C device.
        let mut i2c = I2c::new()?;
        i2c.set_slave_address(address as u16)?;

        let mut ht16k33 = HT16K33::new(i2c, address);
        ht16k33.initialize()?;

        ht16k33.set_display(ht16k33::Display::ON)?;

        Ok(SevenSegment4Display {
            ht16k33: ht16k33,
            light_sensor: light_sensor,
        })
    }

    fn set_brightness(&mut self, brightness: f32) -> Result<(), Error> {
        let level = (brightness * 15.0).round() as u8;
        let dimming = ht16k33::Dimming::from_u8(level)?;

        debug!("Brightness: {}, dimming level: {}/16", brightness, level);

        self.ht16k33.set_dimming(dimming)?;

        Ok(())
    }
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> Display for SevenSegment4Display<'a, T> {
    fn print(
        &mut self,
        time: &DateTime<Local>,
        _: u32,
        _: &Option<OpenWeather>,
    ) -> Result<(), Error> {
        let [d1, d2, d3, d4] = split_time(time)?;
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

        let brightness = self.light_sensor.read_light_normalized()?;
        self.set_brightness(brightness)?;

        Ok(())
    }
}

fn split_time(t: &DateTime<Local>) -> Result<[u8; 4], Error> {
    let hour = t.hour();
    let minute = t.minute();

    let d4 = (minute % 10) as u8;
    let d3 = (minute / 10) as u8 % 10;

    let d2 = (hour % 10) as u8;
    let d1 = (hour / 10) as u8 % 10;

    Ok([d1, d2, d3, d4])
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
            split_time(&Local::now().with_hour(1).unwrap().with_minute(3).unwrap())?,
            [0, 1, 0, 3]
        );
        assert_eq!(
            split_time(&Local::now().with_hour(0).unwrap().with_minute(0).unwrap())?,
            [0, 0, 0, 0]
        );
        assert_eq!(
            split_time(&Local::now().with_hour(12).unwrap().with_minute(34).unwrap())?,
            [1, 2, 3, 4]
        );
        assert_eq!(
            split_time(&Local::now().with_hour(23).unwrap().with_minute(59).unwrap())?,
            [2, 3, 5, 9]
        );

        Ok(())
    }
}
