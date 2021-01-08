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

use std::error::Error;

// To enable heterogenous abstractions
pub enum LightSensorType {
    Fake(FakeLightSensor),
    #[cfg(target_arch = "arm")]
    VEML7700(VEML7700LightSensor),
}

impl LightSensor for LightSensorType {
    fn read_lux(&mut self) -> Result<f32, Box<dyn Error>> {
        match &mut *self {
            Self::Fake(sensor) => sensor.read_lux(),
            #[cfg(target_arch = "arm")]
            Self::VEML7700(sensor) => sensor.read_lux(),
        }
    }
}

pub trait LightSensor {
    fn read_lux(&mut self) -> Result<f32, Box<dyn Error>>;
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

pub struct FakeLightSensor {}

impl FakeLightSensor {
    pub fn new() -> FakeLightSensor {
        FakeLightSensor {}
    }
}

impl LightSensor for FakeLightSensor {
    fn read_lux(&mut self) -> Result<f32, Box<dyn Error>> {
        Ok(100.0)
    }
}
