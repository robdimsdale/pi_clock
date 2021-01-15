use crate::light::LightSensor;
use crate::weather::{OpenWeather, TemperatureUnits};

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
use linux_embedded_hal::I2cdev;
#[cfg(target_arch = "arm")]
use linux_embedded_hal::{Delay, Pin};
#[cfg(target_arch = "arm")]
use rppal::pwm::{Channel, Polarity, Pwm};
#[cfg(target_arch = "arm")]
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

// To enable heterogenous abstractions
pub enum DisplayType<'a, T: LightSensor> {
    Console(ConsoleDisplay<'a, T>),
    #[cfg(target_arch = "arm")]
    HD44780(HD44780Display<'a, T>),
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
        weather: &OpenWeather,
        units: &TemperatureUnits,
    ) {
        match &mut *self {
            Self::Console(display) => display.print(time, weather, units),
            #[cfg(target_arch = "arm")]
            Self::HD44780(display) => display.print(time, weather, units),
            #[cfg(target_arch = "arm")]
            Self::ILI9341(display) => display.print(time, weather, units),
            #[cfg(target_arch = "arm")]
            Self::AlphaNum4(display) => display.print(time, weather, units),
            #[cfg(target_arch = "arm")]
            Self::SevenSegment4(display) => display.print(time, weather, units),
            Self::Composite(displays) => {
                for d in displays.iter_mut() {
                    d.print(time, weather, units);
                }
            }
        }
    }
}

pub trait Display {
    fn print(&mut self, time: &DateTime<Local>, weather: &OpenWeather, units: &TemperatureUnits);
}

pub struct ConsoleDisplay<'a, T: LightSensor> {
    light_sensor: &'a T,
}

impl<'a, T: LightSensor> ConsoleDisplay<'a, T> {
    pub fn new(light_sensor: &'a T) -> ConsoleDisplay<'a, T> {
        ConsoleDisplay {
            light_sensor: light_sensor,
        }
    }
}

impl<'a, T: LightSensor> Display for ConsoleDisplay<'a, T> {
    fn print(&mut self, time: &DateTime<Local>, weather: &OpenWeather, units: &TemperatureUnits) {
        let first_row = format!(
            "{:02}:{:02} {:>10}",
            time.hour(),
            time.minute(),
            truncate_to_characters(&weather.weather[0].main, 7)
        );

        let day = &time.weekday().to_string()[0..3];
        let month = &Month::from_u32(time.month())
            .expect("failed to parse month")
            .name()[0..3];

        // temperature is right-aligned with three characters max (including sign).
        // If the temperature is less than -99° or > 999° we have other problems.
        let second_row = format!(
            "{} {} {:<2} {:>3}°{}",
            day,
            month,
            time.day(),
            &weather.main.temp.round(),
            units.as_char(),
        );
        println!();
        println!("-{}-", std::iter::repeat("-").take(16).collect::<String>());
        println!("|{}|", first_row);
        println!("|{}|", second_row);
        println!("-{}-", std::iter::repeat("-").take(16).collect::<String>());

        println!(
            "Current light: {}",
            self.light_sensor.read_light_normalized().unwrap()
        );
    }
}

#[cfg(target_arch = "arm")]
pub struct HD44780Display<'a, T: LightSensor> {
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
impl<'a, T: LightSensor> HD44780Display<'a, T> {
    pub fn new(light_sensor: &'a T) -> HD44780Display<'a, T> {
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
        )
        .expect("failed to initialize PWM 0 (brightness)");

        pwm0.enable().expect("failed to enable PWM 0 (brightness)");

        rs.export().unwrap();
        en.export().unwrap();
        db4.export().unwrap();
        db5.export().unwrap();
        db6.export().unwrap();
        db7.export().unwrap();
        r.export().unwrap();
        g.export().unwrap();
        b.export().unwrap();

        rs.set_direction(Direction::Low).unwrap();
        en.set_direction(Direction::Low).unwrap();
        db4.set_direction(Direction::Low).unwrap();
        db5.set_direction(Direction::Low).unwrap();
        db6.set_direction(Direction::Low).unwrap();
        db7.set_direction(Direction::Low).unwrap();
        r.set_direction(Direction::Low).unwrap(); // Default to red on; green and blue off
        g.set_direction(Direction::High).unwrap();
        b.set_direction(Direction::High).unwrap();

        let mut lcd = HD44780::new_4bit(rs, en, db4, db5, db6, db7, &mut Delay)
            .expect("failed to create new HD44780");

        lcd.reset(&mut Delay).expect("failed to reset display");
        lcd.clear(&mut Delay).expect("failed to clear display");

        lcd.set_display_mode(
            DisplayMode {
                display: HD44780DisplaySetting::On,
                cursor_visibility: Cursor::Invisible,
                cursor_blink: CursorBlink::Off,
            },
            &mut Delay,
        )
        .expect("failed to set display mode");

        HD44780Display {
            lcd: lcd,
            brightness_pwm: pwm0,
            light_sensor: light_sensor,
        }
    }

    fn set_brightness(&mut self, brightness: f32) {
        println!("Brightness: {}", brightness);

        self.brightness_pwm
            .set_duty_cycle(brightness as f64)
            .expect("failed to set brightness");
    }
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> Display for HD44780Display<'a, T> {
    fn print(&mut self, time: &DateTime<Local>, weather: &OpenWeather, units: &TemperatureUnits) {
        let first_row = format!(
            "{:02}:{:02} {:>10}",
            time.hour(),
            time.minute(),
            truncate_to_characters(&weather.weather[0].main, 7)
        );

        // Move to beginning of first row.
        self.lcd.reset(&mut Delay).expect("failed to reset display");

        self.lcd
            .write_str(&first_row, &mut Delay)
            .expect("failed to write to display");

        // Move to line 2
        self.lcd
            .set_cursor_pos(40, &mut Delay)
            .expect("failed to move to second row");

        let day = &time.weekday().to_string()[0..3];
        let month = &Month::from_u32(time.month())
            .expect("failed to parse month")
            .name()[0..3];

        // temperature is right-aligned with three characters max (including sign).
        // If the temperature is less than -99° or > 999° we have other problems.
        // The X is replaced later with a degree symbol to ensure it is represented as one byte rather than multi-byte (which is what rust will do by default).
        // TODO: can we use b'º' ?
        let second_row = format!(
            "{} {} {:<2} {:>3}X{}",
            day,
            month,
            time.day(),
            &weather.main.temp.round(),
            units.as_char(),
        );
        let mut second_row_as_bytes = second_row.as_bytes().to_vec();
        second_row_as_bytes[14] = 0xDF;

        self.lcd
            .write_bytes(&second_row_as_bytes, &mut Delay)
            .expect("failed to write to display");

        let brightness = self.light_sensor.read_light_normalized().unwrap();
        let min_brightness = 0.01;
        let brightness = brightness.max(min_brightness);

        self.set_brightness(brightness);
    }
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
    pub fn new(light_sensor: &'a T) -> Self {
        let default_brightness = 1.0;
        // pwm0 is pin 18
        let pwm0 = Pwm::with_frequency(
            Channel::Pwm0,
            20000.0,
            default_brightness,
            Polarity::Normal,
            false,
        )
        .expect("failed to initialize PWM 0 (brightness)");
        pwm0.enable().expect("failed to enable PWM 0 (brightness)");

        let rs = Pin::new(24);
        rs.export().unwrap();
        rs.set_direction(Direction::Low).unwrap();

        let cs = Pin::new(21); // TODO: can't use the CE0 pin in the display as it is already used by the SPI variable.
        cs.export().unwrap();
        cs.set_direction(Direction::Low).unwrap();

        let dc = Pin::new(25);
        dc.export().unwrap();
        dc.set_direction(Direction::Low).unwrap();

        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0).unwrap();

        let spi_di = display_interface_spi::SPIInterface::new(spi, dc, cs);

        let mut display = Ili9341::new(spi_di, rs, &mut Delay).unwrap();

        display
            .set_orientation(Orientation::LandscapeFlipped)
            .unwrap();

        ILI9341Display {
            display: display,
            brightness_pwm: pwm0,
            light_sensor: light_sensor,
        }
    }

    fn set_brightness(&mut self, brightness: f32) {
        println!("LED brightness: {}", brightness);

        self.brightness_pwm
            .set_duty_cycle(brightness as f64)
            .expect("failed to set brightness");
    }
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> Display for ILI9341Display<'a, T> {
    fn print(&mut self, time: &DateTime<Local>, weather: &OpenWeather, units: &TemperatureUnits) {
        let day = &time.weekday().to_string()[0..3];
        let month = &Month::from_u32(time.month())
            .expect("failed to parse month")
            .name()[0..3];

        let first_row = format!("{:02}:{:02}", time.hour(), time.minute());

        let second_row = format!("{} {} {:<2}", day, month, time.day());
        let third_row = format!("{}", truncate_to_characters(&weather.weather[0].main, 7));
        let fourth_row = format!("{:>3}°{}", &weather.main.temp.round(), units.as_char());

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

        background.draw(&mut self.display).unwrap();
        time_text.draw(&mut self.display).unwrap();
        other_text.draw(&mut self.display).unwrap();

        let brightness = self.light_sensor.read_light_normalized().unwrap();
        let min_brightness = 0.01;
        let brightness = brightness.max(min_brightness);

        self.set_brightness(brightness);
    }
}

#[cfg(target_arch = "arm")]
pub struct AlphaNum4Display<'a, T: LightSensor> {
    ht16k33: HT16K33<I2cdev>,

    light_sensor: &'a T,
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> AlphaNum4Display<'a, T> {
    pub fn new(light_sensor: &'a T) -> AlphaNum4Display<'a, T> {
        // The I2C device address.
        let address = 0x71;

        // Create an I2C device.
        let mut i2c = I2cdev::new("/dev/i2c-1").unwrap();
        i2c.set_slave_address(address as u16).unwrap();

        let mut ht16k33 = HT16K33::new(i2c, address);
        ht16k33.initialize().unwrap();

        ht16k33.set_display(ht16k33::Display::ON).unwrap();

        AlphaNum4Display {
            ht16k33: ht16k33,
            light_sensor: light_sensor,
        }
    }

    fn set_brightness(&mut self, brightness: f32) {
        let level = (brightness * 15.0).round() as u8;
        let dimming = ht16k33::Dimming::from_u8(level).unwrap();

        println!(
            "Current light level: {}, dimming level: {}/16",
            brightness, level
        );

        self.ht16k33.set_dimming(dimming).unwrap();
    }
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> Display for AlphaNum4Display<'a, T> {
    fn print(&mut self, time: &DateTime<Local>, weather: &OpenWeather, units: &TemperatureUnits) {
        let [d1, d2, d3] = split_temperature(weather.main.temp);
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
            adafruit_alphanum4::AsciiChar::new(units.as_char()),
        );

        self.ht16k33.write_display_buffer().unwrap();

        let brightness = self.light_sensor.read_light_normalized().unwrap();
        self.set_brightness(brightness);
    }
}

#[cfg(target_arch = "arm")]
pub struct SevenSegment4Display<'a, T: LightSensor> {
    ht16k33: HT16K33<I2cdev>,

    light_sensor: &'a T,
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> SevenSegment4Display<'a, T> {
    pub fn new(light_sensor: &'a T) -> SevenSegment4Display<'a, T> {
        // The I2C device address.
        let address = 0x70;

        // Create an I2C device.
        let mut i2c = I2cdev::new("/dev/i2c-1").unwrap();
        i2c.set_slave_address(address as u16).unwrap();

        let mut ht16k33 = HT16K33::new(i2c, address);
        ht16k33.initialize().unwrap();

        ht16k33.set_display(ht16k33::Display::ON).unwrap();

        SevenSegment4Display {
            ht16k33: ht16k33,
            light_sensor: light_sensor,
        }
    }

    fn set_brightness(&mut self, brightness: f32) {
        let level = (brightness * 15.0).round() as u8;
        let dimming = ht16k33::Dimming::from_u8(level).unwrap();

        println!("Brightness: {}, dimming level: {}/16", brightness, level);

        self.ht16k33.set_dimming(dimming).unwrap();
    }
}

#[cfg(target_arch = "arm")]
impl<'a, T: LightSensor> Display for SevenSegment4Display<'a, T> {
    fn print(&mut self, time: &DateTime<Local>, weather: &OpenWeather, units: &TemperatureUnits) {
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
        self.ht16k33.write_display_buffer().unwrap();

        let brightness = self.light_sensor.read_light_normalized().unwrap();
        self.set_brightness(brightness);
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

// If the temperature can be represented with two digits (i.e. 0<=t<=99)
// then leave a gap between the digits and the temperature char
// If the temperature needs three digits (or the negative sign) then skip the gap
// If the temperature is 1 digit then leave a gap either side
// If the temperature is negative 1 digit then add a negative sign before and a gap after
fn split_temperature(temp: f32) -> [char; 3] {
    let is_negative = temp < 0.;
    let zero_char_as_u8 = 48;

    let temp = if is_negative { -temp } else { temp };

    let d3 = ((temp.round() as u16 % 10) as u8 + zero_char_as_u8) as char;
    let d2 = ((temp.round() as u16 / 10) as u8 % 10 + zero_char_as_u8) as char;
    let d1 = ((temp.round() as u16 / 100) as u8 % 10 + zero_char_as_u8) as char;

    let (d1, d2, d3) = if (is_negative && temp >= 10.0) || temp > 100. {
        (d1, d2, d3)
    } else {
        (d2, d3, ' ')
    };
    let d1 = if is_negative { '-' } else { d1 };

    let d1 = if d1 == '0' { ' ' } else { d1 };

    // TODO: Explore degree symbol instead of space
    // To emulate a degree symbol use the following:
    // 0b XNMLKJIHGGFEDCBA>
    // 0b X000000011100011

    [d1, d2, d3]
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
    fn test_split_temperature() {
        assert_eq!(split_temperature(46.0), ['4', '6', ' ']);
        assert_eq!(split_temperature(123.0), ['1', '2', '3']);
        assert_eq!(split_temperature(123.4), ['1', '2', '3']);
        assert_eq!(split_temperature(275.4), ['2', '7', '5']);
        assert_eq!(split_temperature(1.4), [' ', '1', ' ']);
        assert_eq!(split_temperature(-1.4), ['-', '1', ' ']);
        assert_eq!(split_temperature(-12.4), ['-', '1', '2']);
        // assert_eq!(split_temperature(-123.4), ['-', '1', '2']); // TODO: should panic
    }

    #[test]
    fn test_split_time() {
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
    }
}
