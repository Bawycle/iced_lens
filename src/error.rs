// SPDX-License-Identifier: MPL-2.0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_io_error() {
        let err = Error::Io("disk failure".to_string());
        assert_eq!(format!("{}", err), "I/O Error: disk failure");
    }

    #[test]
    fn from_io_error_produces_io_variant() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "boom");
        let err: Error = io_error.into();
        match err {
            Error::Io(message) => assert!(message.contains("boom")),
            _ => panic!("expected Io variant"),
        }
    }

    #[test]
    fn svg_error_from_string() {
        let err: Error = "invalid svg data".to_string().into();
        match err {
            Error::Svg(message) => assert!(message.contains("invalid svg")),
            _ => panic!("expected Svg variant"),
        }
    }
}
