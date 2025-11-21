use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    Io(String),
    Svg(String),
    // Add other error variants as needed
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O Error: {}", e),
            Error::Svg(e) => write!(f, "SVG Error: {}", e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
