use rand::prelude::*;
#[cfg(target_arch = "arm")]
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

    #[cfg(target_arch = "arm")]
    I2C(rppal::i2c::Error),

    #[cfg(target_arch = "arm")]
    VEML(veml6030::Error<rppal::i2c::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::LockRng => write!(f, "a task failed while holding RNG lock"),

            ErrorKind::LockLightSensor => {
                write!(f, "a task failed while holding Light Sensor lock")
            }

            #[cfg(target_arch = "arm")]
            ErrorKind::I2C(ref err) => err.fmt(f),

            #[cfg(target_arch = "arm")]
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

#[cfg(target_arch = "arm")]
impl From<PoisonError<MutexGuard<'_, veml6030::Veml6030<I2c>>>> for Error {
    fn from(_: PoisonError<MutexGuard<'_, veml6030::Veml6030<I2c>>>) -> Self {
        Error {
            kind: ErrorKind::LockLightSensor,
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
impl From<veml6030::Error<rppal::i2c::Error>> for Error {
    fn from(e: veml6030::Error<rppal::i2c::Error>) -> Self {
        Error {
            kind: ErrorKind::VEML(e),
        }
    }
}
