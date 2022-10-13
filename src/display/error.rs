use crate::light::Error as LightError;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl std::error::Error for Error {}

impl Error {
    /// Return the kind of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

/// The kind of an error that can occur.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    LightSensor(LightError),

    #[cfg(feature = "rpi-hw")]
    I2C(rppal::i2c::Error),

    #[cfg(feature = "rpi-hw")]
    Pwm(rppal::pwm::Error),

    #[cfg(feature = "rpi-hw")]
    Spi(rppal::spi::Error),

    #[cfg(feature = "rpi-hw")]
    Gpio(linux_embedded_hal::sysfs_gpio::Error),

    #[cfg(feature = "rpi-hw")]
    HT16K33(ht16k33::ValidationError),

    #[cfg(feature = "rpi-hw")]
    HD44780(hd44780_driver::error::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::LightSensor(ref err) => err.fmt(f),

            #[cfg(feature = "rpi-hw")]
            ErrorKind::I2C(ref err) => err.fmt(f),

            #[cfg(feature = "rpi-hw")]
            ErrorKind::Pwm(ref err) => err.fmt(f),

            #[cfg(feature = "rpi-hw")]
            ErrorKind::Spi(ref err) => err.fmt(f),

            #[cfg(feature = "rpi-hw")]
            ErrorKind::Gpio(ref err) => err.fmt(f),

            #[cfg(feature = "rpi-hw")]
            ErrorKind::HT16K33(ref err) => err.fmt(f),

            #[cfg(feature = "rpi-hw")]
            ErrorKind::HD44780(ref err) => write!(f, "{:?}", err),
        }
    }
}

impl From<LightError> for Error {
    fn from(e: LightError) -> Self {
        Error {
            kind: ErrorKind::LightSensor(e),
        }
    }
}

#[cfg(feature = "rpi-hw")]
impl From<rppal::i2c::Error> for Error {
    fn from(e: rppal::i2c::Error) -> Self {
        Error {
            kind: ErrorKind::I2C(e),
        }
    }
}

#[cfg(feature = "rpi-hw")]
impl From<linux_embedded_hal::sysfs_gpio::Error> for Error {
    fn from(e: linux_embedded_hal::sysfs_gpio::Error) -> Self {
        Error {
            kind: ErrorKind::Gpio(e),
        }
    }
}

#[cfg(feature = "rpi-hw")]
impl From<rppal::pwm::Error> for Error {
    fn from(e: rppal::pwm::Error) -> Self {
        Error {
            kind: ErrorKind::Pwm(e),
        }
    }
}

#[cfg(feature = "rpi-hw")]
impl From<rppal::spi::Error> for Error {
    fn from(e: rppal::spi::Error) -> Self {
        Error {
            kind: ErrorKind::Spi(e),
        }
    }
}

#[cfg(feature = "rpi-hw")]
impl From<ht16k33::ValidationError> for Error {
    fn from(e: ht16k33::ValidationError) -> Self {
        Error {
            kind: ErrorKind::HT16K33(e),
        }
    }
}

#[cfg(feature = "rpi-hw")]
impl From<hd44780_driver::error::Error> for Error {
    fn from(e: hd44780_driver::error::Error) -> Self {
        Error {
            kind: ErrorKind::HD44780(e),
        }
    }
}
