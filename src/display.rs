use crate::light::{LightSensor, LightSensorType};
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
use ili9341::{Ili9341, Orientation};
#[cfg(target_arch = "arm")]
use linux_embedded_hal::sysfs_gpio::Direction;
#[cfg(target_arch = "arm")]
use linux_embedded_hal::{Delay, Pin};
#[cfg(target_arch = "arm")]
use rppal::pwm::{Channel, Polarity, Pwm};
#[cfg(target_arch = "arm")]
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

// To enable heterogenous abstractions
pub enum DisplayType<'a> {
    Console(ConsoleDisplay<'a>),
    #[cfg(target_arch = "arm")]
    HD44780(HD44780Display),
    #[cfg(target_arch = "arm")]
    ILI9341(ILI9341Display),
}

impl<'a> DisplayType<'a> {
    pub fn print(
        &mut self,
        time: &DateTime<Local>,
        weather: &OpenWeather,
        units: &TemperatureUnits,
    ) {
        match &mut *self {
            Self::Console(console_display) => console_display.print(time, weather, units),
            #[cfg(target_arch = "arm")]
            Self::HD44780(hd44780_display) => hd44780_display.print(time, weather, units),
            #[cfg(target_arch = "arm")]
            Self::ILI9341(ili9341_display) => ili9341_display.print(time, weather, units),
        }
    }
}

pub trait Display {
    fn print(&mut self, time: &DateTime<Local>, weather: &OpenWeather, units: &TemperatureUnits);
}

pub struct ConsoleDisplay<'a> {
    light_sensor: &'a mut LightSensorType,
}

impl<'a> ConsoleDisplay<'a> {
    pub fn new(light_sensor: &'a mut LightSensorType) -> ConsoleDisplay<'a> {
        ConsoleDisplay {
            light_sensor: light_sensor,
        }
    }
}

impl<'a> Display for ConsoleDisplay<'a> {
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

        println!("Current light: {}", self.light_sensor.read_lux().unwrap());
    }
}

#[cfg(target_arch = "arm")]
pub struct HD44780Display {
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

#[cfg(target_arch = "arm")]
impl HD44780Display {
    pub fn new(brightness: f64) -> HD44780Display {
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

        // pwm0 is pin 18
        let pwm0 = Pwm::with_frequency(Channel::Pwm0, 20000.0, brightness, Polarity::Normal, false)
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
        }
    }

    pub fn set_brightness(&mut self, brightness: f64) {
        // TODO: Validate 0 <= brightness <= 1
        self.brightness_pwm
            .set_duty_cycle(brightness)
            .expect("failed to set brightness");
    }
}

#[cfg(target_arch = "arm")]
impl Display for HD44780Display {
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
    }
}

#[cfg(target_arch = "arm")]
pub struct ILI9341Display {
    display: Ili9341<
        display_interface_spi::SPIInterface<Spi, linux_embedded_hal::Pin, linux_embedded_hal::Pin>,
        linux_embedded_hal::Pin,
    >,
    brightness_pwm: Pwm,
}

#[cfg(target_arch = "arm")]
impl ILI9341Display {
    pub fn new(brightness: f64) -> Self {
        // pwm0 is pin 18
        let pwm0 = Pwm::with_frequency(Channel::Pwm0, 20000.0, brightness, Polarity::Normal, false)
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
        }
    }

    pub fn set_brightness(&mut self, brightness: f64) {
        // TODO: Validate 0 <= brightness <= 1
        self.brightness_pwm
            .set_duty_cycle(brightness)
            .expect("failed to set brightness");
    }
}

#[cfg(target_arch = "arm")]
impl Display for ILI9341Display {
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
    }
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
}
