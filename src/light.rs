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

use chrono::{Local, NaiveTime};
use rand::prelude::*;
use std::error::Error;

// To enable heterogenous abstractions
pub enum LightSensorType {
    Random(RandomLightSensor),
    Time(TimeLightSensor),
    #[cfg(target_arch = "arm")]
    VEML7700(VEML7700LightSensor),
}

impl LightSensor for LightSensorType {
    fn read_lux(&mut self) -> Result<f32, Box<dyn Error>> {
        match &mut *self {
            Self::Random(sensor) => sensor.read_lux(),
            Self::Time(sensor) => sensor.read_lux(),
            #[cfg(target_arch = "arm")]
            Self::VEML7700(sensor) => sensor.read_lux(),
        }
    }
}

pub trait LightSensor {
    fn read_lux(&mut self) -> Result<f32, Box<dyn Error>>;
}

pub struct TimeLightSensor {}

impl TimeLightSensor {
    pub fn new() -> TimeLightSensor {
        TimeLightSensor {}
    }
}

impl LightSensor for TimeLightSensor {
    fn read_lux(&mut self) -> Result<f32, Box<dyn Error>> {
        let now = Local::now();

        let time_low = NaiveTime::from_hms(8, 0, 0);
        let time_high = NaiveTime::from_hms(22, 30, 0);
        let range = time_low..time_high;

        if range.contains(&now.time()) {
            return Ok(1000.0);
        } else {
            return Ok(1.0);
        }
    }
}

#[cfg(target_arch = "arm")]
pub struct VEML7700LightSensor {}

#[cfg(target_arch = "arm")]
impl VEML7700LightSensor {
    pub fn new() -> VEML7700LightSensor {
        VEML7700LightSensor {}
    }
}

#[cfg(target_arch = "arm")]
impl LightSensor for VEML7700LightSensor {
    fn read_lux(&mut self) -> Result<f32, Box<dyn Error>> {
        return Ok(100.0);
    }
}

pub struct RandomLightSensor {
    rng: ThreadRng,
}

impl RandomLightSensor {
    pub fn new() -> RandomLightSensor {
        RandomLightSensor { rng: thread_rng() }
    }
}

impl LightSensor for RandomLightSensor {
    fn read_lux(&mut self) -> Result<f32, Box<dyn Error>> {
        let val = self.rng.gen_range(1.0..1000.0);
        Ok(val)
    }
}
