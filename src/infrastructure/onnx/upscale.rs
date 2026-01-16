// SPDX-License-Identifier: MPL-2.0
//! `Real-ESRGAN` upscale adapter implementing the [`AIProcessor`] trait.
//!
//! [`AIProcessor`]: crate::application::port::AIProcessor

use std::sync::{Arc, Mutex};

use crate::application::port::{AIError, AIProcessor, ProcessorCapabilities};
use crate::domain::media::RawImage;
use crate::media::upscale::{UpscaleManager, UPSCALE_FACTOR};

/// ONNX-based upscale processor using `Real-ESRGAN` x4.
///
/// This adapter wraps [`UpscaleManager`] to implement the [`AIProcessor`] trait.
/// It converts between domain types ([`RawImage`]) and image crate types internally.
///
/// # Thread Safety
///
/// This type is `Send + Sync` via internal locking.
///
/// # Example
///
/// ```ignore
/// use iced_lens::infrastructure::onnx::OnnxUpscaleProcessor;
/// use iced_lens::application::port::AIProcessor;
///
/// let processor = OnnxUpscaleProcessor::new();
/// if processor.is_ready() {
///     let upscaled = processor.process(&image)?;
///     // Result is 4x the original dimensions
/// }
/// ```
pub struct OnnxUpscaleProcessor {
    /// The underlying upscale manager (wrapped for thread safety).
    manager: Arc<Mutex<UpscaleManager>>,
}

impl Default for OnnxUpscaleProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl OnnxUpscaleProcessor {
    /// Creates a new `Real-ESRGAN` x4 upscale processor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(UpscaleManager::new())),
        }
    }

    /// Creates a processor from an existing `UpscaleManager`.
    ///
    /// This is useful when integrating with existing code that already
    /// manages the upscale model lifecycle.
    #[must_use]
    pub fn from_manager(manager: Arc<Mutex<UpscaleManager>>) -> Self {
        Self { manager }
    }

    /// Returns a reference to the underlying manager.
    ///
    /// Use this for model lifecycle operations (download, validation).
    #[must_use]
    pub fn manager(&self) -> &Arc<Mutex<UpscaleManager>> {
        &self.manager
    }

    /// Loads the model session.
    ///
    /// # Errors
    ///
    /// Returns an error if the model is not downloaded or loading fails.
    pub fn load(&self) -> Result<(), AIError> {
        let mut manager = self
            .manager
            .lock()
            .map_err(|_| AIError::ProcessingFailed("Lock poisoned".to_string()))?;

        manager
            .load_session(None)
            .map_err(|e| AIError::ModelLoadFailed(e.to_string()))
    }
}

impl AIProcessor for OnnxUpscaleProcessor {
    fn process(&self, image: &RawImage) -> Result<RawImage, AIError> {
        let mut manager = self
            .manager
            .lock()
            .map_err(|_| AIError::ProcessingFailed("Lock poisoned".to_string()))?;

        // Convert RawImage (RGBA) to DynamicImage (RGB)
        let dynamic_image = raw_image_to_dynamic(image)?;

        // Run upscale
        let result = manager
            .upscale(&dynamic_image)
            .map_err(|e| AIError::InferenceFailed(e.to_string()))?;

        // Convert back to RawImage
        Ok(dynamic_to_raw_image(&result))
    }

    fn is_ready(&self) -> bool {
        self.manager
            .lock()
            .map(|m| m.is_session_ready())
            .unwrap_or(false)
    }

    fn capabilities(&self) -> ProcessorCapabilities {
        // NAFNet and Real-ESRGAN are proper model names
        #[allow(clippy::doc_markdown)]
        ProcessorCapabilities::upscaler("Real-ESRGAN x4", UPSCALE_FACTOR)
            .with_input_limit(1920, 1080) // 4K output limit
            .with_time_factor(2.0)
    }
}

// SAFETY: The inner Mutex provides the necessary synchronization.
// UpscaleManager is Send, and Mutex<T: Send> is Sync.
unsafe impl Sync for OnnxUpscaleProcessor {}

/// Converts a `RawImage` (RGBA) to a `DynamicImage` (RGB).
fn raw_image_to_dynamic(raw: &RawImage) -> Result<image_rs::DynamicImage, AIError> {
    let rgba_data = raw.rgba_bytes_arc();
    let width = raw.width();
    let height = raw.height();

    // Create RGBA image from raw bytes
    let rgba_image = image_rs::RgbaImage::from_raw(width, height, rgba_data.to_vec())
        .ok_or_else(|| AIError::ProcessingFailed("Failed to create RGBA image".to_string()))?;

    // Convert to DynamicImage
    Ok(image_rs::DynamicImage::ImageRgba8(rgba_image))
}

/// Converts a `DynamicImage` to a `RawImage` (RGBA).
fn dynamic_to_raw_image(image: &image_rs::DynamicImage) -> RawImage {
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let data = rgba.into_raw();

    RawImage::new(width, height, Arc::new(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processor_can_be_created() {
        let processor = OnnxUpscaleProcessor::new();
        assert!(!processor.is_ready()); // Model not loaded by default
    }

    #[test]
    fn processor_default_is_same_as_new() {
        let processor = OnnxUpscaleProcessor::default();
        assert!(!processor.is_ready());
    }

    #[test]
    fn capabilities_are_correct() {
        let processor = OnnxUpscaleProcessor::new();
        let caps = processor.capabilities();

        assert_eq!(caps.name, "Real-ESRGAN x4");
        assert!(caps.is_upscaler());
        assert_eq!(caps.scale_factor, Some(4));
        assert!(caps.input_size_limit.is_some());
    }

    #[test]
    fn raw_image_conversion_roundtrip() {
        // Create a small test image
        let width = 10;
        let height = 10;
        #[allow(clippy::cast_possible_truncation)] // Test data: i % 256 fits in u8
        let data: Vec<u8> = (0..width * height * 4).map(|i| (i % 256) as u8).collect();
        let raw = RawImage::new(width, height, Arc::new(data.clone()));

        // Convert to DynamicImage and back
        let dynamic = raw_image_to_dynamic(&raw).expect("should convert to dynamic");
        let back = dynamic_to_raw_image(&dynamic);

        assert_eq!(back.width(), width);
        assert_eq!(back.height(), height);
    }

    #[test]
    fn process_fails_when_not_ready() {
        let processor = OnnxUpscaleProcessor::new();
        let image = RawImage::new(10, 10, Arc::new(vec![0u8; 400]));

        let result = processor.process(&image);
        assert!(result.is_err());
    }
}
