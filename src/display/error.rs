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
pub enum ErrorKind {
    LightSensor(LightError),

    #[cfg(target_arch = "arm")]
    I2C(rppal::i2c::Error),

    #[cfg(target_arch = "arm")]
    PWM(rppal::pwm::Error),

    #[cfg(target_arch = "arm")]
    SPI(rppal::spi::Error),

    #[cfg(target_arch = "arm")]
    GPIO(linux_embedded_hal::sysfs_gpio::Error),

    #[cfg(target_arch = "arm")]
    HT16K33(ht16k33::ValidationError),

    #[cfg(target_arch = "arm")]
    ILI9341(ili9341::Error<linux_embedded_hal::sysfs_gpio::Error>),

    #[cfg(target_arch = "arm")]
    HD44780(hd44780_driver::error::Error),

    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::LightSensor(ref err) => err.fmt(f),

            #[cfg(target_arch = "arm")]
            ErrorKind::I2C(ref err) => err.fmt(f),

            #[cfg(target_arch = "arm")]
            ErrorKind::PWM(ref err) => err.fmt(f),

            #[cfg(target_arch = "arm")]
            ErrorKind::SPI(ref err) => err.fmt(f),

            #[cfg(target_arch = "arm")]
            ErrorKind::GPIO(ref err) => err.fmt(f),

            #[cfg(target_arch = "arm")]
            ErrorKind::HT16K33(ref err) => err.fmt(f),

            #[cfg(target_arch = "arm")]
            ErrorKind::ILI9341(ref err) => write!(f, "{:?}", err),

            #[cfg(target_arch = "arm")]
            ErrorKind::HD44780(ref err) => write!(f, "{:?}", err),

            ErrorKind::__Nonexhaustive => unreachable!(),
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

#[cfg(target_arch = "arm")]
impl From<rppal::i2c::Error> for Error {
    fn from(e: rppal::i2c::Error) -> Self {
        Error {
            kind: ErrorKind::I2C(e),
        }
    }
}

#[cfg(target_arch = "arm")]
impl From<linux_embedded_hal::sysfs_gpio::Error> for Error {
    fn from(e: linux_embedded_hal::sysfs_gpio::Error) -> Self {
        Error {
            kind: ErrorKind::GPIO(e),
        }
    }
}

#[cfg(target_arch = "arm")]
impl From<rppal::pwm::Error> for Error {
    fn from(e: rppal::pwm::Error) -> Self {
        Error {
            kind: ErrorKind::PWM(e),
        }
    }
}

#[cfg(target_arch = "arm")]
impl From<rppal::spi::Error> for Error {
    fn from(e: rppal::spi::Error) -> Self {
        Error {
            kind: ErrorKind::SPI(e),
        }
    }
}

#[cfg(target_arch = "arm")]
impl From<ht16k33::ValidationError> for Error {
    fn from(e: ht16k33::ValidationError) -> Self {
        Error {
            kind: ErrorKind::HT16K33(e),
        }
    }
}

#[cfg(target_arch = "arm")]
impl From<ili9341::Error<linux_embedded_hal::sysfs_gpio::Error>> for Error {
    fn from(e: ili9341::Error<linux_embedded_hal::sysfs_gpio::Error>) -> Self {
        Error {
            kind: ErrorKind::ILI9341(e),
        }
    }
}

#[cfg(target_arch = "arm")]
impl From<hd44780_driver::error::Error> for Error {
    fn from(e: hd44780_driver::error::Error) -> Self {
        Error {
            kind: ErrorKind::HD44780(e),
        }
    }
}
