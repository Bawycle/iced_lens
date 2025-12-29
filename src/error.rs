// SPDX-License-Identifier: MPL-2.0
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    Io(String),
    Svg(String),
    Config(String),
    Video(VideoError),
}

/// Specific error types for video playback issues.
/// Used to provide user-friendly, localized error messages.
#[derive(Debug, Clone)]
pub enum VideoError {
    /// File format is not supported (e.g., unknown extension)
    UnsupportedFormat,

    /// Video codec is not supported by the system's `FFmpeg`
    UnsupportedCodec(String),

    /// File appears corrupted or has invalid data
    CorruptedFile,

    /// File exists but contains no video stream
    NoVideoStream,

    /// Decoding failed during playback
    DecodingFailed(String),

    /// I/O error (file not found, permission denied, etc.)
    IoError(String),

    /// Decoder thread terminated unexpectedly
    DecoderDied,

    /// Seek operation timed out (target may be beyond EOF)
    SeekTimeout,

    /// Generic error with raw message
    Other(String),
}

impl VideoError {
    /// Returns the i18n message key for this error type.
    #[must_use]
    pub fn i18n_key(&self) -> &'static str {
        match self {
            VideoError::UnsupportedFormat => "error-load-video-unsupported-format",
            VideoError::UnsupportedCodec(_) => "error-load-video-unsupported-codec",
            VideoError::CorruptedFile => "error-load-video-corrupted",
            VideoError::NoVideoStream => "error-load-video-no-video-stream",
            VideoError::DecodingFailed(_) => "error-load-video-decoding-failed",
            VideoError::IoError(_) => "error-load-video-io",
            VideoError::DecoderDied => "error-video-decoder-died",
            VideoError::SeekTimeout => "error-video-seek-timeout",
            VideoError::Other(_) => "error-load-video-general",
        }
    }

    /// Returns the i18n variable arguments for this error type.
    ///
    /// Some error messages contain placeholders like `{ $codec }` or `{ $message }`
    /// that need to be filled with runtime values.
    #[must_use] 
    pub fn i18n_args(&self) -> Vec<(&'static str, String)> {
        match self {
            VideoError::UnsupportedCodec(codec) => vec![("codec", codec.clone())],
            VideoError::DecodingFailed(msg) => vec![("message", msg.clone())],
            _ => vec![],
        }
    }

    /// Attempts to parse a raw error message into a specific `VideoError` type.
    /// This is used to categorize errors from FFmpeg/decoder.
    #[must_use] 
    pub fn from_message(msg: &str) -> Self {
        let msg_lower = msg.to_lowercase();

        // Check codec/decoder errors first (they might contain "not found")
        if msg_lower.contains("codec") || msg_lower.contains("decoder") {
            // Try to extract codec name
            if let Some(codec) = Self::extract_codec_name(&msg_lower) {
                return VideoError::UnsupportedCodec(codec);
            } else if msg_lower.contains("not found") || msg_lower.contains("unsupported") {
                return VideoError::DecodingFailed(msg.to_string());
            }
        }

        // I/O errors (file access issues)
        if msg_lower.contains("no such file")
            || (msg_lower.contains("not found") && !msg_lower.contains("decoder"))
            || msg_lower.contains("permission denied")
            || msg_lower.contains("i/o error")
        {
            return VideoError::IoError(msg.to_string());
        }

        // No video stream
        if msg_lower.contains("no video stream")
            || msg_lower.contains("no video track")
            || msg_lower.contains("invalid data found")
        {
            return VideoError::NoVideoStream;
        }

        // Corrupted file
        if msg_lower.contains("corrupt")
            || msg_lower.contains("invalid")
            || msg_lower.contains("malformed")
        {
            return VideoError::CorruptedFile;
        }

        // Seek timeout (specific pattern from decoder)
        if msg_lower.contains("seek timeout") {
            return VideoError::SeekTimeout;
        }

        // Decoder died (channel closed unexpectedly)
        if msg_lower.contains("decoder died")
            || msg_lower.contains("decoder stopped")
            || msg_lower.contains("decoder not running")
        {
            return VideoError::DecoderDied;
        }

        // Decoding failures
        if msg_lower.contains("packet")
            || msg_lower.contains("scaling")
            || msg_lower.contains("seek")
            || msg_lower.contains("decode")
            || msg_lower.contains("unsupported")
        {
            return VideoError::DecodingFailed(msg.to_string());
        }

        VideoError::Other(msg.to_string())
    }

    /// Tries to extract a codec name from an error message.
    fn extract_codec_name(msg: &str) -> Option<String> {
        // Common patterns: "codec 'xyz' not found", "decoder xyz not found"
        let codecs = [
            "h264", "hevc", "h265", "vp8", "vp9", "av1", "mpeg4", "mpeg2",
        ];
        for codec in codecs {
            if msg.contains(codec) {
                return Some(codec.to_uppercase());
            }
        }
        None
    }
}

impl fmt::Display for VideoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoError::UnsupportedFormat => write!(f, "Unsupported video format"),
            VideoError::UnsupportedCodec(codec) => {
                write!(f, "Unsupported video codec: {codec}")
            }
            VideoError::CorruptedFile => write!(f, "Video file is corrupted"),
            VideoError::NoVideoStream => write!(f, "No video stream found"),
            VideoError::DecodingFailed(msg) => write!(f, "Decoding failed: {msg}"),
            VideoError::IoError(msg) => write!(f, "I/O error: {msg}"),
            VideoError::DecoderDied => write!(f, "Video decoder stopped unexpectedly"),
            VideoError::SeekTimeout => write!(f, "Seek operation timed out"),
            VideoError::Other(msg) => write!(f, "{msg}"),
        }
    }
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
    fn svg_error_from_string() {
        let err: Error = "invalid svg data".to_string().into();
        match err {
            Error::Svg(message) => assert!(message.contains("invalid svg")),
            _ => panic!("expected Svg variant"),
        }
    }

    #[test]
    fn config_error_formats_properly() {
        let err = Error::Config("bad field".into());
        assert_eq!(format!("{err}"), "Config Error: bad field");
    }

    #[test]
    fn video_error_from_message_io() {
        let err = VideoError::from_message("No such file or directory");
        assert!(matches!(err, VideoError::IoError(_)));
    }

    #[test]
    fn video_error_from_message_no_stream() {
        let err = VideoError::from_message("No video stream found in file");
        assert!(matches!(err, VideoError::NoVideoStream));
    }

    #[test]
    fn video_error_from_message_codec() {
        let err = VideoError::from_message("Decoder h264 not found");
        assert!(matches!(err, VideoError::UnsupportedCodec(codec) if codec == "H264"));
    }

    #[test]
    fn video_error_from_message_corrupted() {
        let err = VideoError::from_message("File is corrupt or invalid");
        assert!(matches!(err, VideoError::CorruptedFile));
    }

    #[test]
    fn video_error_from_message_decoding() {
        let err = VideoError::from_message("Packet send failed: error");
        assert!(matches!(err, VideoError::DecodingFailed(_)));
    }

    #[test]
    fn video_error_i18n_keys() {
        assert_eq!(
            VideoError::UnsupportedFormat.i18n_key(),
            "error-load-video-unsupported-format"
        );
        assert_eq!(
            VideoError::CorruptedFile.i18n_key(),
            "error-load-video-corrupted"
        );
        assert_eq!(
            VideoError::NoVideoStream.i18n_key(),
            "error-load-video-no-video-stream"
        );
    }

    #[test]
    fn video_error_display() {
        let err = VideoError::UnsupportedCodec("H264".to_string());
        assert!(format!("{err}").contains("H264"));
    }

    #[test]
    fn video_error_i18n_args() {
        // UnsupportedCodec returns codec arg
        let codec_err = VideoError::UnsupportedCodec("H264".to_string());
        let args = codec_err.i18n_args();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], ("codec", "H264".to_string()));

        // DecodingFailed returns message arg
        let decode_err = VideoError::DecodingFailed("packet error".to_string());
        let args = decode_err.i18n_args();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], ("message", "packet error".to_string()));

        // Other variants return empty args
        assert!(VideoError::UnsupportedFormat.i18n_args().is_empty());
        assert!(VideoError::CorruptedFile.i18n_args().is_empty());
        assert!(VideoError::NoVideoStream.i18n_args().is_empty());
        assert!(VideoError::DecoderDied.i18n_args().is_empty());
        assert!(VideoError::SeekTimeout.i18n_args().is_empty());
    }

    #[test]
    fn video_error_from_message_seek_timeout() {
        let err = VideoError::from_message("Seek timeout: target may be beyond end of file");
        assert!(matches!(err, VideoError::SeekTimeout));
    }

    #[test]
    fn video_error_from_message_decoder_died() {
        let err = VideoError::from_message("Video decoder stopped unexpectedly");
        assert!(matches!(err, VideoError::DecoderDied));
    }

    #[test]
    fn video_error_decoder_died_i18n_key() {
        assert_eq!(
            VideoError::DecoderDied.i18n_key(),
            "error-video-decoder-died"
        );
    }

    #[test]
    fn video_error_seek_timeout_i18n_key() {
        assert_eq!(
            VideoError::SeekTimeout.i18n_key(),
            "error-video-seek-timeout"
        );
    }

    #[test]
    fn video_error_decoder_died_display() {
        let err = VideoError::DecoderDied;
        assert!(format!("{err}").contains("stopped unexpectedly"));
    }

    #[test]
    fn video_error_seek_timeout_display() {
        let err = VideoError::SeekTimeout;
        assert!(format!("{err}").contains("timed out"));
    }
}
