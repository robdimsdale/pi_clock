use rand::prelude::*;
#[cfg(feature = "rpi-hw")]
use rppal::i2c::I2c;
use std::fmt;
use std::sync::{MutexGuard, PoisonError};

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
    LockRng,

    LockLightSensor,

    #[cfg(feature = "rpi-hw")]
    I2C(rppal::i2c::Error),

    #[cfg(feature = "rpi-hw")]
    VEML(veml6030::Error<rppal::i2c::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::LockRng => write!(f, "a task failed while holding RNG lock"),

            ErrorKind::LockLightSensor => {
                write!(f, "a task failed while holding Light Sensor lock")
            }

            #[cfg(feature = "rpi-hw")]
            ErrorKind::I2C(ref err) => err.fmt(f),

            #[cfg(feature = "rpi-hw")]
            ErrorKind::VEML(ref err) => write!(f, "{:?}", err),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, ThreadRng>>> for Error {
    fn from(_: PoisonError<MutexGuard<'_, ThreadRng>>) -> Self {
        Error {
            kind: ErrorKind::LockRng,
        }
    }
}

#[cfg(feature = "rpi-hw")]
impl From<PoisonError<MutexGuard<'_, veml6030::Veml6030<I2c>>>> for Error {
    fn from(_: PoisonError<MutexGuard<'_, veml6030::Veml6030<I2c>>>) -> Self {
        Error {
            kind: ErrorKind::LockLightSensor,
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
impl From<veml6030::Error<rppal::i2c::Error>> for Error {
    fn from(e: veml6030::Error<rppal::i2c::Error>) -> Self {
        Error {
            kind: ErrorKind::VEML(e),
        }
    }
}
