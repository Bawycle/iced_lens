// SPDX-License-Identifier: MPL-2.0
//! AI processing port definition.
//!
//! This module defines the [`AIProcessor`] trait for AI-based image processing
//! operations like deblurring and upscaling.
//!
//! # Design Notes
//!
//! - Progress callbacks are handled via Iced Messages, not in the trait
//! - Model downloading and validation is implementation-specific
//! - The trait is `Send + Sync` for thread-safe concurrent processing

use crate::domain::media::RawImage;
use std::fmt;

// =============================================================================
// AIError
// =============================================================================

/// Errors that can occur during AI processing.
#[derive(Debug, Clone)]
pub enum AIError {
    /// The AI model is not ready (not downloaded or not validated).
    ModelNotReady,

    /// Processing failed with an error message.
    ProcessingFailed(String),

    /// The input image is too large for the model.
    InputTooLarge {
        /// Maximum supported dimensions.
        max: (u32, u32),
        /// Actual image dimensions.
        actual: (u32, u32),
    },

    /// The model file could not be loaded.
    ModelLoadFailed(String),

    /// Inference failed during processing.
    InferenceFailed(String),
}

impl fmt::Display for AIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AIError::ModelNotReady => write!(f, "AI model is not ready"),
            AIError::ProcessingFailed(msg) => write!(f, "AI processing failed: {msg}"),
            AIError::InputTooLarge { max, actual } => {
                write!(
                    f,
                    "Input too large: {}x{} (max: {}x{})",
                    actual.0, actual.1, max.0, max.1
                )
            }
            AIError::ModelLoadFailed(msg) => write!(f, "Failed to load model: {msg}"),
            AIError::InferenceFailed(msg) => write!(f, "Inference failed: {msg}"),
        }
    }
}

impl std::error::Error for AIError {}

// =============================================================================
// ProcessorCapabilities
// =============================================================================

/// Describes the capabilities of an AI processor.
///
/// This is used to query what a processor can do before invoking it.
#[derive(Debug, Clone)]
pub struct ProcessorCapabilities {
    // Allow: NAFNet and Real-ESRGAN are proper model names, not code identifiers.
    #[allow(clippy::doc_markdown)]
    /// Human-readable name of the processor (e.g., "NAFNet Deblur", "Real-ESRGAN x4").
    pub name: &'static str,

    /// Scale factor for upscaling processors (e.g., 4 for Real-ESRGAN x4).
    /// `None` for processors that don't change image size (e.g., deblur).
    pub scale_factor: Option<u32>,

    /// Maximum input dimensions supported by the model.
    /// `None` if there's no practical limit.
    pub input_size_limit: Option<(u32, u32)>,

    /// Estimated processing time factor (1.0 = baseline, 2.0 = twice as slow).
    /// This is a rough estimate for UI progress indication.
    pub time_factor: f32,
}

impl ProcessorCapabilities {
    /// Creates capabilities for a processor that doesn't scale (e.g., deblur).
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            scale_factor: None,
            input_size_limit: None,
            time_factor: 1.0,
        }
    }

    /// Creates capabilities for an upscaling processor.
    #[must_use]
    pub const fn upscaler(name: &'static str, scale_factor: u32) -> Self {
        Self {
            name,
            scale_factor: Some(scale_factor),
            input_size_limit: None,
            time_factor: 1.0,
        }
    }

    /// Sets the input size limit.
    #[must_use]
    pub const fn with_input_limit(mut self, max_width: u32, max_height: u32) -> Self {
        self.input_size_limit = Some((max_width, max_height));
        self
    }

    /// Sets the time factor.
    #[must_use]
    pub const fn with_time_factor(mut self, factor: f32) -> Self {
        self.time_factor = factor;
        self
    }

    /// Returns `true` if this processor is an upscaler.
    #[must_use]
    pub fn is_upscaler(&self) -> bool {
        self.scale_factor.is_some()
    }

    /// Checks if the given dimensions are within the input size limit.
    #[must_use]
    pub fn supports_size(&self, width: u32, height: u32) -> bool {
        match self.input_size_limit {
            Some((max_w, max_h)) => width <= max_w && height <= max_h,
            None => true,
        }
    }
}

// =============================================================================
// AIProcessor Trait
// =============================================================================

/// Port for AI-based image processing.
///
/// This trait defines the interface for AI processors like deblurring and
/// upscaling models. Infrastructure adapters implement this trait using
/// ONNX Runtime or other ML frameworks.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for concurrent processing.
///
/// # Example
///
/// ```ignore
/// use iced_lens::application::port::ai::{AIProcessor, AIError};
/// use iced_lens::domain::media::RawImage;
///
/// fn enhance_image(processor: &impl AIProcessor, image: &RawImage) -> Result<RawImage, AIError> {
///     // Check if processor is ready
///     if !processor.is_ready() {
///         return Err(AIError::ModelNotReady);
///     }
///
///     // Check size limits
///     let caps = processor.capabilities();
///     if !caps.supports_size(image.width(), image.height()) {
///         return Err(AIError::InputTooLarge {
///             max: caps.input_size_limit.unwrap(),
///             actual: (image.width(), image.height()),
///         });
///     }
///
///     // Process
///     processor.process(image)
/// }
/// ```
pub trait AIProcessor: Send + Sync {
    /// Processes an image through the AI model.
    ///
    /// For deblurring, returns a deblurred image of the same size.
    /// For upscaling, returns an upscaled image (size multiplied by scale factor).
    ///
    /// # Errors
    ///
    /// Returns an [`AIError`] if:
    /// - The model is not ready
    /// - The input is too large
    /// - Processing fails
    fn process(&self, image: &RawImage) -> Result<RawImage, AIError>;

    /// Checks if the model is ready for processing.
    ///
    /// A model is ready when it has been downloaded, validated, and loaded.
    fn is_ready(&self) -> bool;

    /// Returns the capabilities of this processor.
    fn capabilities(&self) -> ProcessorCapabilities;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn ai_error_display() {
        let err = AIError::ModelNotReady;
        assert_eq!(format!("{err}"), "AI model is not ready");

        let err = AIError::ProcessingFailed("out of memory".to_string());
        assert!(format!("{err}").contains("out of memory"));

        let err = AIError::InputTooLarge {
            max: (1024, 1024),
            actual: (2048, 2048),
        };
        let display = format!("{err}");
        assert!(display.contains("2048x2048"));
        assert!(display.contains("1024x1024"));
    }

    #[test]
    fn processor_capabilities_basic() {
        let caps = ProcessorCapabilities::new("Test Deblur");
        assert_eq!(caps.name, "Test Deblur");
        assert!(caps.scale_factor.is_none());
        assert!(!caps.is_upscaler());
        assert!(caps.supports_size(10000, 10000)); // No limit
    }

    #[test]
    fn processor_capabilities_upscaler() {
        let caps = ProcessorCapabilities::upscaler("Real-ESRGAN x4", 4)
            .with_input_limit(1024, 1024)
            .with_time_factor(2.0);

        assert_eq!(caps.name, "Real-ESRGAN x4");
        assert_eq!(caps.scale_factor, Some(4));
        assert!(caps.is_upscaler());
        assert!(caps.supports_size(512, 512));
        assert!(!caps.supports_size(2048, 2048));
        assert!((caps.time_factor - 2.0).abs() < f32::EPSILON);
    }

    // Mock implementation for testing
    struct MockProcessor {
        ready: bool,
    }

    impl AIProcessor for MockProcessor {
        fn process(&self, image: &RawImage) -> Result<RawImage, AIError> {
            if !self.ready {
                return Err(AIError::ModelNotReady);
            }
            // Return same image (identity transform for testing)
            Ok(RawImage::new(
                image.width(),
                image.height(),
                image.rgba_bytes_arc(),
            ))
        }

        fn is_ready(&self) -> bool {
            self.ready
        }

        fn capabilities(&self) -> ProcessorCapabilities {
            ProcessorCapabilities::new("Mock Processor")
        }
    }

    #[test]
    fn mock_processor_ready() {
        let processor = MockProcessor { ready: true };
        assert!(processor.is_ready());

        let image = RawImage::new(10, 10, Arc::new(vec![0u8; 400]));
        let result = processor.process(&image);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_processor_not_ready() {
        let processor = MockProcessor { ready: false };
        assert!(!processor.is_ready());

        let image = RawImage::new(10, 10, Arc::new(vec![0u8; 400]));
        let result = processor.process(&image);
        assert!(matches!(result, Err(AIError::ModelNotReady)));
    }
}
