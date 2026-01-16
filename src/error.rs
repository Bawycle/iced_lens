// SPDX-License-Identifier: MPL-2.0
use std::fmt;

// Re-export domain error type
pub use crate::domain::error::VideoError;

#[derive(Debug, Clone)]
pub enum Error {
    Io(String),
    Svg(String),
    Config(String),
    Video(VideoError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O Error: {e}"),
            Error::Svg(e) => write!(f, "SVG Error: {e}"),
            Error::Config(e) => write!(f, "Config Error: {e}"),
            Error::Video(e) => write!(f, "Video Error: {e}"),
        }
    }
}

impl From<VideoError> for Error {
    fn from(err: VideoError) -> Self {
        Error::Video(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Config(err.to_string())
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::Config(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_io_error() {
        let err = Error::Io("disk failure".to_string());
        assert_eq!(format!("{err}"), "I/O Error: disk failure");
    }

    #[test]
    fn from_io_error_produces_io_variant() {
        let io_error = std::io::Error::other("boom");
        let err: Error = io_error.into();
        match err {
            Error::Io(message) => assert!(message.contains("boom")),
            _ => panic!("expected Io variant"),
        }
    }

    #[test]
    fn svg_error_formats_properly() {
        let err = Error::Svg("invalid svg data".to_string());
        assert!(format!("{err}").contains("invalid svg"));
    }

    #[test]
    fn config_error_formats_properly() {
        let err = Error::Config("bad field".into());
        assert_eq!(format!("{err}"), "Config Error: bad field");
    }

    #[test]
    fn video_error_converts_to_error() {
        let video_err = VideoError::UnsupportedFormat;
        let err: Error = video_err.into();
        assert!(matches!(err, Error::Video(VideoError::UnsupportedFormat)));
    }

    #[test]
    fn video_error_display_through_error() {
        let err = Error::Video(VideoError::UnsupportedCodec("H264".to_string()));
        assert!(format!("{err}").contains("H264"));
    }
}
