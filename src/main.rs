#[cfg(target_arch = "arm")]
use linux_embedded_hal::sysfs_gpio::Direction;
#[cfg(target_arch = "arm")]
use linux_embedded_hal::{Delay, Pin};

use chrono::{Datelike, Local, Month, Timelike};
#[cfg(target_arch = "arm")]
use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode, HD44780};
use num_traits::cast::FromPrimitive;
#[cfg(target_arch = "arm")]
use rppal::pwm::{Channel, Polarity, Pwm};
use std::env;
use std::{thread, time};

#[cfg(target_arch = "arm")]
use embedded_graphics::{
    egrectangle, egtext, fonts::Font12x16, fonts::Font24x32, pixelcolor::Rgb565, prelude::*,
    primitive_style, text_style,
};
#[cfg(target_arch = "arm")]
use ili9341::{Ili9341, Orientation};
#[cfg(target_arch = "arm")]
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

const DEFAULT_BRIGHTNESS: f64 = 0.05;
const OPEN_WEATHER_API_KEY_VAR: &'static str = "OPEN_WEATHER_API_KEY";
const LAT_VAR: &'static str = "LAT";
const LON_VAR: &'static str = "LON";
const UNITS_VAR: &'static str = "UNIT";

const DEFAULT_UNITS: &'static str = "imperial";

#[cfg(not(target_arch = "arm"))]
fn main() {
    println!("Initializing");

    let open_weather_api_key = env::var(OPEN_WEATHER_API_KEY_VAR).expect(&format!(
        "Must provide {} env var",
        OPEN_WEATHER_API_KEY_VAR
    ));
    let lat = env::var(LAT_VAR).expect(&format!("Must provide {} env var", LAT_VAR));
    let lon = env::var(LON_VAR).expect(&format!("Must provide {} env var", LON_VAR));
    let units = env::var(UNITS_VAR).unwrap_or(DEFAULT_UNITS.to_owned());

    println!("Initialization complete");

    println!("Initialization complete");

    let mut last_weather_attempt = time::Instant::now();
    let mut last_weather_success = time::Instant::now();

    let mut weather = pi_clock::get_weather(&open_weather_api_key, &lat, &lon, &units)
        .expect("failed to get initial weather");

    loop {
        let now = time::Instant::now();

        let duration_since_last_weather = now.duration_since(last_weather_attempt);
        if duration_since_last_weather > time::Duration::from_secs(600) {
            last_weather_attempt = now;

            println!(
                "Getting updated weather ({}s since last attempt)",
                duration_since_last_weather.as_secs(),
            );

            if let Ok(updated_weather) =
                pi_clock::get_weather(&open_weather_api_key, &lat, &lon, &units)
            {
                println!("successfully updated weather");

                last_weather_success = last_weather_attempt;
                weather = updated_weather
            } else {
                println!(
                    "failed to update weather (using previous weather). {}s since last success",
                    now.duration_since(last_weather_success).as_secs()
                );
            }
        }

        let temp = &weather.main.temp;

        let now = Local::now();
        let first_row = format!(
            "{:02}:{:02} {:>10}",
            now.hour(),
            now.minute(),
            truncate_to_characters(&weather.weather[0].main, 7)
        );

        let day = &now.weekday().to_string()[0..3];
        let month = &Month::from_u32(now.month())
            .expect("failed to parse month")
            .name()[0..3];

        // temperature is right-aligned with three characters max (including sign).
        // If the temperature is less than -99°F or > 999°F we have other problems.
        let second_row = format!("{} {} {:<2} {:>3}°F", day, month, now.day(), temp.round());
        println!();
        println!("-{}-", std::iter::repeat("-").take(16).collect::<String>());
        println!("|{}|", first_row);
        println!("|{}|", second_row);
        println!("-{}-", std::iter::repeat("-").take(16).collect::<String>());

        thread::sleep(time::Duration::from_secs(1));
    }
}

#[cfg(target_arch = "arm")]
fn main() {
    println!("Initializing");

    let args: Vec<String> = env::args().collect();

    let brightness = match args.len() {
        0..=1 => DEFAULT_BRIGHTNESS,
        _ => (&args[1]).parse().unwrap_or(DEFAULT_BRIGHTNESS),
    };

    let open_weather_api_key = env::var(OPEN_WEATHER_API_KEY_VAR).expect(&format!(
        "Must provide {} env var",
        OPEN_WEATHER_API_KEY_VAR
    ));
    let lat = env::var(LAT_VAR).expect(&format!("Must provide {} env var", LAT_VAR));
    let lon = env::var(LON_VAR).expect(&format!("Must provide {} env var", LON_VAR));
    let units = env::var(UNITS_VAR).unwrap_or(DEFAULT_UNITS.to_owned());

    println!("Setting up peripherals");

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

    println!("Initialization complete");

    let mut last_weather_attempt = time::Instant::now();
    let mut last_weather_success = time::Instant::now();

    let mut weather = pi_clock::get_weather(&open_weather_api_key, &lat, &lon, &units)
        .expect("failed to get initial weather");

    loop {
        let now = time::Instant::now();

        let duration_since_last_weather = now.duration_since(last_weather_attempt);
        if duration_since_last_weather > time::Duration::from_secs(600) {
            last_weather_attempt = now;

            println!(
                "Getting updated weather ({}s since last attempt)",
                duration_since_last_weather.as_secs(),
            );

            if let Ok(updated_weather) =
                pi_clock::get_weather(&open_weather_api_key, &lat, &lon, &units)
            {
                println!("successfully updated weather");

                last_weather_success = last_weather_attempt;
                weather = updated_weather
            } else {
                println!(
                    "failed to update weather (using previous weather). {}s since last success",
                    now.duration_since(last_weather_success).as_secs()
                );
            }
        }

        let now = Local::now();

        let day = &now.weekday().to_string()[0..3];
        let month = &Month::from_u32(now.month())
            .expect("failed to parse month")
            .name()[0..3];

        let temp = &weather.main.temp;

        let first_row = format!("{:02}:{:02}", now.hour(), now.minute());

        let second_row = format!("{} {} {:<2}", day, month, now.day());
        let third_row = format!("{}", truncate_to_characters(&weather.weather[0].main, 7));
        let fourth_row = format!("{:>3}°F", temp.round());

        let text = format!("{}\n{}\n{}", second_row, third_row, fourth_row);

        let r = egrectangle!(
            top_left = (0, 0),
            bottom_right = (320, 240),
            style = primitive_style!(fill_color = Rgb565::BLACK),
        );

        let t_time = egtext!(
            text = &first_row,
            top_left = (20, 16),
            style = text_style!(font = Font24x32, text_color = Rgb565::RED),
        );

        let t = egtext!(
            text = &text,
            top_left = (20, 48),
            style = text_style!(font = Font12x16, text_color = Rgb565::RED),
        );

        // println!("Drawing black rectangle (background)");
        r.draw(&mut display).unwrap();

        // println!("Drawing time text");
        t_time.draw(&mut display).unwrap();

        // println!("Drawing remaining text");
        t.draw(&mut display).unwrap();

        // println!("Drawing complete - sleeping");

        thread::sleep(time::Duration::from_secs(5));
    }

    // // Using BCM numbers
    // // i.e. pin 0 corresponds to wiringpi 30 and physical 27

    // let rs = Pin::new(21);
    // let en = Pin::new(20);
    // let db4 = Pin::new(26);
    // let db5 = Pin::new(13);
    // let db6 = Pin::new(6);
    // let db7 = Pin::new(5);
    // let r = Pin::new(17);
    // let g = Pin::new(16);
    // let b = Pin::new(19);

    // // pwm0 is pin 18
    // let pwm0 = Pwm::with_frequency(Channel::Pwm0, 20000.0, brightness, Polarity::Normal, false)
    //     .expect("failed to initialize PWM 0 (brightness)");

    // pwm0.enable().expect("failed to enable PWM 0 (brightness)");

    // rs.export().unwrap();
    // en.export().unwrap();
    // db4.export().unwrap();
    // db5.export().unwrap();
    // db6.export().unwrap();
    // db7.export().unwrap();
    // r.export().unwrap();
    // g.export().unwrap();
    // b.export().unwrap();

    // rs.set_direction(Direction::Low).unwrap();
    // en.set_direction(Direction::Low).unwrap();
    // db4.set_direction(Direction::Low).unwrap();
    // db5.set_direction(Direction::Low).unwrap();
    // db6.set_direction(Direction::Low).unwrap();
    // db7.set_direction(Direction::Low).unwrap();
    // r.set_direction(Direction::Low).unwrap(); // Default to red on; green and blue off
    // g.set_direction(Direction::High).unwrap();
    // b.set_direction(Direction::High).unwrap();

    // let mut lcd = HD44780::new_4bit(rs, en, db4, db5, db6, db7, &mut Delay)
    //     .expect("failed to create new HD44780");

    // lcd.reset(&mut Delay).expect("failed to reset display");
    // lcd.clear(&mut Delay).expect("failed to clear display");

    // lcd.set_display_mode(
    //     DisplayMode {
    //         display: Display::On,
    //         cursor_visibility: Cursor::Invisible,
    //         cursor_blink: CursorBlink::Off,
    //     },
    //     &mut Delay,
    // )
    // .expect("failed to set display mode");

    // println!("Initialization complete");

    // let mut last_weather_attempt = time::Instant::now();
    // let mut last_weather_success = time::Instant::now();

    // let mut weather = pi_clock::get_weather(&open_weather_api_key, &lat, &lon, &units)
    //     .expect("failed to get initial weather");

    // loop {
    //     let now = time::Instant::now();

    //     let duration_since_last_weather = now.duration_since(last_weather_attempt);
    //     if duration_since_last_weather > time::Duration::from_secs(600) {
    //         last_weather_attempt = now;

    //         println!(
    //             "Getting updated weather ({}s since last attempt)",
    //             duration_since_last_weather.as_secs(),
    //         );

    //         if let Ok(updated_weather) =
    //             pi_clock::get_weather(&open_weather_api_key, &lat, &lon, &units)
    //         {
    //             println!("successfully updated weather");

    //             last_weather_success = last_weather_attempt;
    //             weather = updated_weather
    //         } else {
    //             println!(
    //                 "failed to update weather (using previous weather). {}s since last success",
    //                 now.duration_since(last_weather_success).as_secs()
    //             );
    //         }
    //     }

    //     let temp = &weather.main.temp;

    //     let now = Local::now();
    //     let first_row = format!(
    //         "{:02}:{:02} {:>10}",
    //         now.hour(),
    //         now.minute(),
    //         truncate_to_characters(&weather.weather[0].main, 7)
    //     );

    //     // Move to beginning of first row.
    //     lcd.reset(&mut Delay).expect("failed to reset display");

    //     lcd.write_str(&first_row, &mut Delay)
    //         .expect("failed to write to display");

    //     // Move to line 2
    //     lcd.set_cursor_pos(40, &mut Delay)
    //         .expect("failed to move to second row");

    //     let day = &now.weekday().to_string()[0..3];
    //     let month = &Month::from_u32(now.month())
    //         .expect("failed to parse month")
    //         .name()[0..3];

    //     // temperature is right-aligned with three characters max (including sign).
    //     // If the temperature is less than -99°F or > 999°F we have other problems.
    //     // The X is replaced later with a degree symbol to ensure it is represented as one byte rather than multi-byte (which is what rust will do by default).
    //     let second_row = format!("{} {} {:<2} {:>3}XF", day, month, now.day(), temp.round());
    //     let mut second_row_as_bytes = second_row.as_bytes().to_vec();
    //     second_row_as_bytes[14] = 0xDF;

    //     lcd.write_bytes(&second_row_as_bytes, &mut Delay)
    //         .expect("failed to write to display");

    //     thread::sleep(time::Duration::from_secs(1));
    // }
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
