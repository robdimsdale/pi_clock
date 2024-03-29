mod error;

// From: https://en.wikipedia.org/wiki/Lux
//
// Illuminance (lux)	Surfaces illuminated by
//
// 0.0001	            Moonless, overcast night sky (starlight)
// 0.002	            Moonless clear night sky with airglow
// 0.05–0.3	            Full moon on a clear night
// 3.4	                Dark limit of civil twilight under a clear sky
// 20–50	            Public areas with dark surroundings
// 50	                Family living room lights
// 80	                Office building hallway/toilet lighting
// 100	                Very dark overcast day
// 150	                Train station platforms
// 320–500	            Office lighting
// 400	                Sunrise or sunset on a clear day.
// 1000	                Overcast day; typical TV studio lighting
// 10,000–25,000	    Full daylight (not direct sun)
// 32,000–100,000	    Direct sunlight

pub use error::Error;

use chrono::{Local, NaiveTime};
use lazy_static::*;
use rand::prelude::*;
use std::sync::Mutex;

#[cfg(feature = "rpi-hw")]
use log::debug;
#[cfg(feature = "rpi-hw")]
use rppal::i2c::I2c;
#[cfg(feature = "rpi-hw")]
use veml6030::{SlaveAddr, Veml6030};

const MAX_LUX: f32 = 1.0;
const MIN_LUX: f32 = 0.01;

lazy_static! {
    static ref MAX_LUX_START_TIME: NaiveTime = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    static ref MAX_LUX_END_TIME: NaiveTime = NaiveTime::from_hms_opt(19, 0, 0).unwrap();
    static ref MIN_LUX_START_TIME: NaiveTime = NaiveTime::from_hms_opt(23, 0, 0).unwrap(); // Must be before midnight
    static ref MIN_LUX_END_TIME: NaiveTime = NaiveTime::from_hms_opt(7, 0, 0).unwrap(); // Must be after midnight
}

// To enable heterogenous abstractions
pub enum LightSensorType {
    Random(RandomLightSensor),
    Time(TimeLightSensor),
    #[cfg(feature = "rpi-hw")]
    VEML7700(VEML7700LightSensor), // TODO: consider add caching here to avoid lots of mutexes
}

impl LightSensor for LightSensorType {
    fn read_light_normalized(&self) -> Result<f32, Error> {
        match &self {
            Self::Random(sensor) => sensor.read_light_normalized(),
            Self::Time(sensor) => sensor.read_light_normalized(),
            #[cfg(feature = "rpi-hw")]
            Self::VEML7700(sensor) => sensor.read_light_normalized(),
        }
    }
}

// Returns a value between 0 and 1
pub trait LightSensor {
    fn read_light_normalized(&self) -> Result<f32, Error>;
}

pub struct TimeLightSensor {}

impl TimeLightSensor {
    pub fn new() -> TimeLightSensor {
        TimeLightSensor {}
    }
}

impl Default for TimeLightSensor {
    fn default() -> Self {
        TimeLightSensor::new()
    }
}

impl LightSensor for TimeLightSensor {
    fn read_light_normalized(&self) -> Result<f32, Error> {
        Ok(time_based_brightness_for_time(&Local::now().time()))
    }
}

fn time_based_brightness_for_time(t: &NaiveTime) -> f32 {
    let midnight = NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap();

    let full_bright_range = *MAX_LUX_START_TIME..*MAX_LUX_END_TIME;
    let bright_to_dark_range = *MAX_LUX_END_TIME..*MIN_LUX_START_TIME;
    let full_dark_range1 = *MIN_LUX_START_TIME..(midnight - chrono::TimeDelta::nanoseconds(1));
    let full_dark_range2 = midnight..*MIN_LUX_END_TIME;
    let dark_to_bright_range = *MIN_LUX_END_TIME..*MAX_LUX_START_TIME;

    // Separate case for end-of-day bound as ranges are exxclusive
    if *t == midnight - chrono::TimeDelta::nanoseconds(1) {
        return 0.;
    }

    if full_bright_range.contains(t) {
        return 1.;
    }

    if full_dark_range1.contains(t) || full_dark_range2.contains(t) {
        return 0.;
    }

    if bright_to_dark_range.contains(t) {
        let time_since_full_bright = t.signed_duration_since(*MAX_LUX_END_TIME);
        let time_until_full_dark = MIN_LUX_START_TIME.signed_duration_since(*t);

        let progress = time_until_full_dark.num_seconds() as f32
            / (time_since_full_bright.num_seconds() as f32
                + time_until_full_dark.num_seconds() as f32);
        let brightness = progress * (MAX_LUX - MIN_LUX) + MIN_LUX;

        return normalize_lux(brightness);
    }

    if dark_to_bright_range.contains(t) {
        let time_since_full_dark = t.signed_duration_since(*MIN_LUX_END_TIME);
        let time_until_full_bright = MAX_LUX_START_TIME.signed_duration_since(*t);

        let progress = time_since_full_dark.num_seconds() as f32
            / (time_since_full_dark.num_seconds() as f32
                + time_until_full_bright.num_seconds() as f32);

        let brightness = progress * (MAX_LUX - MIN_LUX) + MIN_LUX;

        return normalize_lux(brightness);
    }

    panic!("Bad time bounds!")
}

#[cfg(feature = "rpi-hw")]
pub struct VEML7700LightSensor {
    sensor: Mutex<Veml6030<I2c>>,
}

#[cfg(feature = "rpi-hw")]
impl VEML7700LightSensor {
    pub fn new() -> Result<Self, Error> {
        let i2c = I2c::new()?;
        let mut sensor = Veml6030::new(i2c, SlaveAddr::default());
        sensor.enable()?;

        Ok(VEML7700LightSensor {
            sensor: Mutex::new(sensor),
        })
    }
}

#[cfg(feature = "rpi-hw")]
impl LightSensor for VEML7700LightSensor {
    fn read_light_normalized(&self) -> Result<f32, Error> {
        let lux = self.sensor.lock()?.read_lux()?;
        debug!("Lux: {}", lux);

        Ok(normalize_lux(lux))
    }
}

pub struct RandomLightSensor {
    rng: Mutex<ThreadRng>,
}

impl RandomLightSensor {
    pub fn new() -> RandomLightSensor {
        RandomLightSensor {
            rng: Mutex::new(thread_rng()),
        }
    }
}

impl Default for RandomLightSensor {
    fn default() -> Self {
        RandomLightSensor::new()
    }
}

impl LightSensor for RandomLightSensor {
    fn read_light_normalized(&self) -> Result<f32, Error> {
        let val = self.rng.lock()?.gen_range(MIN_LUX..MAX_LUX);
        Ok(normalize_lux(val))
    }
}

// Return a value between 0 and 1, truncating to MIN_LUX or MAX_LUX as necessary.
fn normalize_lux(lux: f32) -> f32 {
    let truncated = lux.clamp(MIN_LUX, MAX_LUX);
    (truncated - MIN_LUX) / (MAX_LUX - MIN_LUX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round(number: f32, places: i32) -> f32 {
        let factor = 10_f32.powi(places);
        (number * factor).round() / factor
    }

    #[test]
    fn test_normalize_lux() {
        assert_eq!(normalize_lux(MIN_LUX), 0.);
        assert_eq!(normalize_lux(MIN_LUX - 1.0), 0.);
        assert_eq!(normalize_lux(MAX_LUX), 1.);
        assert_eq!(normalize_lux(MAX_LUX + 1.0), 1.);
    }

    #[test]
    fn test_time_based_brightness_for_time() {
        // Full brightness

        assert_eq!(
            round(time_based_brightness_for_time(&(*MAX_LUX_START_TIME)), 0),
            1.
        );

        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(*MAX_LUX_START_TIME + chrono::TimeDelta::nanoseconds(1))
                ),
                0
            ),
            1.
        );

        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(*MAX_LUX_END_TIME - chrono::TimeDelta::nanoseconds(1))
                ),
                0
            ),
            1.
        );

        // Scaling from brightness to darkness

        assert_eq!(
            round(time_based_brightness_for_time(&(*MAX_LUX_END_TIME)), 0),
            1.
        );

        // A time close enough to the end of full brightness will round to full brightness
        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(*MAX_LUX_END_TIME + chrono::TimeDelta::milliseconds(1))
                ),
                0
            ),
            1.,
        );

        let quarter_bright_to_dark =
            *MAX_LUX_END_TIME + (*MIN_LUX_START_TIME - *MAX_LUX_END_TIME) / 4;

        assert_eq!(
            round(time_based_brightness_for_time(&quarter_bright_to_dark), 2),
            0.75,
        );

        let mid_bright_to_dark = *MAX_LUX_END_TIME + (*MIN_LUX_START_TIME - *MAX_LUX_END_TIME) / 2;

        assert_eq!(
            round(time_based_brightness_for_time(&mid_bright_to_dark), 1),
            0.5,
        );

        let three_quarter_bright_to_dark =
            *MAX_LUX_END_TIME + ((*MIN_LUX_START_TIME - *MAX_LUX_END_TIME) * 3) / 4;

        assert_eq!(
            round(
                time_based_brightness_for_time(&three_quarter_bright_to_dark),
                2
            ),
            0.25,
        );

        // A time close enough to the start of full darkness will round to full darkness
        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(*MIN_LUX_START_TIME - chrono::TimeDelta::milliseconds(1))
                ),
                0
            ),
            0.,
        );

        // Full Darkness

        assert_eq!(
            round(time_based_brightness_for_time(&(*MIN_LUX_START_TIME)), 0),
            0.
        );

        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(*MIN_LUX_START_TIME + chrono::TimeDelta::nanoseconds(1))
                ),
                0
            ),
            0.
        );

        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(*MIN_LUX_END_TIME - chrono::TimeDelta::nanoseconds(1))
                ),
                0
            ),
            0.
        );

        // Midnight bounds for full darkness

        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap())
                ),
                0
            ),
            0.
        );

        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap()
                        - chrono::TimeDelta::nanoseconds(1))
                ),
                0
            ),
            0.
        );

        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap()
                        - chrono::TimeDelta::nanoseconds(2))
                ),
                0
            ),
            0.
        );

        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap()
                        + chrono::TimeDelta::nanoseconds(1))
                ),
                0
            ),
            0.
        );

        // Scaling from darkness to brightness

        assert_eq!(
            round(time_based_brightness_for_time(&(*MIN_LUX_END_TIME)), 0),
            0.
        );

        // A time close enough to the end of full darkness will round to full darkness
        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(*MIN_LUX_END_TIME + chrono::TimeDelta::milliseconds(1))
                ),
                0
            ),
            0.,
        );

        let quarter_dark_to_bright =
            *MIN_LUX_END_TIME + (*MAX_LUX_START_TIME - *MIN_LUX_END_TIME) / 4;

        assert_eq!(
            round(time_based_brightness_for_time(&quarter_dark_to_bright), 2),
            0.25,
        );

        let mid_dark_to_bright = *MIN_LUX_END_TIME + (*MAX_LUX_START_TIME - *MIN_LUX_END_TIME) / 2;

        assert_eq!(
            round(time_based_brightness_for_time(&mid_dark_to_bright), 1),
            0.5,
        );

        let three_quarter_dark_to_bright =
            *MIN_LUX_END_TIME + ((*MAX_LUX_START_TIME - *MIN_LUX_END_TIME) * 3) / 4;

        assert_eq!(
            round(
                time_based_brightness_for_time(&three_quarter_dark_to_bright),
                2
            ),
            0.75,
        );

        // A time close enough to the start of full brightness will round to full darkness
        assert_eq!(
            round(
                time_based_brightness_for_time(
                    &(*MAX_LUX_START_TIME - chrono::TimeDelta::milliseconds(1))
                ),
                0
            ),
            1.,
        );
    }
}
