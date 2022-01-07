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
    Http(ureq::Error),
    StringParse(std::io::Error),
    JSONParse(serde_json::Error),
    Transport(ureq::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Http(ref err) => err.fmt(f),
            ErrorKind::StringParse(ref err) => err.fmt(f),
            ErrorKind::JSONParse(ref err) => err.fmt(f),
            ErrorKind::Transport(ref err) => err.fmt(f),
        }
    }
}

impl From<ureq::Error> for Error {
    fn from(e: ureq::Error) -> Self {
        match e {
            ureq::Error::Status(_, _) => Error {
                kind: ErrorKind::Http(e),
            },
            ureq::Error::Transport(_) => Error {
                kind: ErrorKind::Transport(e),
            },
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error {
            kind: ErrorKind::StringParse(e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error {
            kind: ErrorKind::JSONParse(e),
        }
    }
}
