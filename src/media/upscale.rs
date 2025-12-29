// SPDX-License-Identifier: MPL-2.0
//! AI-powered image upscaling using Real-ESRGAN ONNX model.
//!
//! This module provides functionality for:
//! - Downloading the Real-ESRGAN ONNX model from a configurable URL
//! - Verifying model integrity with BLAKE3 checksum
//! - Running inference to upscale images by 4x
//!
//! # Upscaling Strategy
//!
//! Real-ESRGAN provides fixed 4x upscaling. For intermediate scale factors:
//! - Apply 4x Real-ESRGAN upscaling first
//! - Then downscale to target dimensions using Lanczos3 interpolation
//!
//! This produces better quality than direct interpolation for enlargements.

use crate::app::paths;

/// Filename for the downloaded upscale model in the data directory.
const MODEL_FILENAME: &str = "realesrgan-x4plus.onnx";

use image_rs::DynamicImage;
use ndarray::Array4;
use ort::session::{builder::GraphOptimizationLevel, Session};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Cancellation token type for background tasks.
pub type CancellationToken = Arc<AtomicBool>;

/// Checks if the cancellation token has been triggered.
#[inline]
pub fn is_cancelled(token: &CancellationToken) -> bool {
    token.load(Ordering::SeqCst)
}

/// Result type for upscale operations.
pub type UpscaleResult<T> = Result<T, UpscaleError>;

/// Errors that can occur during upscaling operations.
#[derive(Debug, Clone)]
pub enum UpscaleError {
    /// Model file not found at expected path.
    ModelNotFound,
    /// Failed to download the model.
    DownloadFailed(String),
    /// Model checksum verification failed.
    ChecksumMismatch { expected: String, actual: String },
    /// ONNX inference failed.
    InferenceFailed(String),
    /// Image preprocessing failed.
    PreprocessingFailed(String),
    /// Image postprocessing failed.
    PostprocessingFailed(String),
    /// Operation was cancelled by user.
    Cancelled,
    /// IO error occurred.
    Io(String),
    /// Model session not initialized.
    SessionNotInitialized,
}

impl std::fmt::Display for UpscaleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpscaleError::ModelNotFound => write!(f, "Model file not found"),
            UpscaleError::DownloadFailed(msg) => write!(f, "Download failed: {msg}"),
            UpscaleError::ChecksumMismatch { expected, actual } => {
                write!(f, "Checksum mismatch: expected {expected}, got {actual}")
            }
            UpscaleError::InferenceFailed(msg) => write!(f, "Inference failed: {msg}"),
            UpscaleError::PreprocessingFailed(msg) => write!(f, "Preprocessing failed: {msg}"),
            UpscaleError::PostprocessingFailed(msg) => write!(f, "Postprocessing failed: {msg}"),
            UpscaleError::Cancelled => write!(f, "Operation cancelled"),
            UpscaleError::Io(msg) => write!(f, "IO error: {msg}"),
            UpscaleError::SessionNotInitialized => write!(f, "ONNX session not initialized"),
        }
    }
}

impl std::error::Error for UpscaleError {}

/// Status of the upscale model.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum UpscaleModelStatus {
    /// Model has not been downloaded.
    #[default]
    NotDownloaded,
    /// Model is currently being downloaded.
    Downloading { progress: f32 },
    /// Model is being validated (checksum + test inference).
    Validating,
    /// Model is ready for use.
    Ready,
    /// An error occurred.
    Error(String),
}

/// The fixed upscale factor provided by Real-ESRGAN x4plus model.
pub const UPSCALE_FACTOR: u32 = 4;

/// Manager for the Real-ESRGAN upscaling model.
///
/// Handles model lifecycle: download, validation, and inference.
pub struct UpscaleManager {
    model_path: PathBuf,
    session: Option<Session>,
}

impl Default for UpscaleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UpscaleManager {
    /// Creates a new `UpscaleManager` instance.
    #[must_use] 
    pub fn new() -> Self {
        let model_path = get_model_path();
        Self {
            model_path,
            session: None,
        }
    }

    /// Returns the path where the model is/will be stored.
    #[must_use] 
    pub fn model_path(&self) -> &PathBuf {
        &self.model_path
    }

    /// Checks if the model file exists on disk.
    #[must_use] 
    pub fn is_model_downloaded(&self) -> bool {
        self.model_path.exists()
    }

    /// Loads the ONNX session from the model file.
    ///
    /// Must be called after the model is downloaded and verified.
    /// If a cancellation token is provided and triggered, returns `UpscaleError::Cancelled`.
    ///
    /// # Errors
    ///
    /// Returns an error if the model file is not found, the operation is cancelled,
    /// or the ONNX session fails to initialize.
    pub fn load_session(&mut self, cancel_token: Option<&CancellationToken>) -> UpscaleResult<()> {
        // Check for cancellation before loading
        if let Some(token) = cancel_token {
            if is_cancelled(token) {
                return Err(UpscaleError::Cancelled);
            }
        }

        if !self.model_path.exists() {
            return Err(UpscaleError::ModelNotFound);
        }

        let session = Session::builder()
            .map_err(|e| UpscaleError::InferenceFailed(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| UpscaleError::InferenceFailed(e.to_string()))?
            .commit_from_file(&self.model_path)
            .map_err(|e| UpscaleError::InferenceFailed(e.to_string()))?;

        self.session = Some(session);
        Ok(())
    }

    /// Checks if the ONNX session is loaded and ready.
    #[must_use] 
    pub fn is_session_ready(&self) -> bool {
        self.session.is_some()
    }

    /// Runs 4x upscaling inference on an image.
    ///
    /// Returns the upscaled image (4x the original dimensions).
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not initialized, preprocessing fails,
    /// or the ONNX inference fails.
    pub fn upscale(&mut self, image: &DynamicImage) -> UpscaleResult<DynamicImage> {
        let session = self
            .session
            .as_mut()
            .ok_or(UpscaleError::SessionNotInitialized)?;

        // Preprocess: DynamicImage -> NCHW tensor (RGB, normalized 0-1)
        let input_tensor = preprocess_image(image)?;

        // Ensure standard layout for ONNX Runtime
        let input_tensor = input_tensor.as_standard_layout().into_owned();

        // Get input name from model (Real-ESRGAN typically uses 'input')
        let input_name = session
            .inputs
            .first()
            .map_or_else(|| "input".to_string(), |i| i.name.clone());

        // Create tensor reference for inference
        let input_ref = ort::value::TensorRef::from_array_view(&input_tensor)
            .map_err(|e| UpscaleError::InferenceFailed(e.to_string()))?;

        let outputs = session
            .run(ort::inputs![input_name.as_str() => input_ref])
            .map_err(|e| UpscaleError::InferenceFailed(e.to_string()))?;

        // Postprocess: NCHW tensor -> DynamicImage
        postprocess_output(&outputs)
    }

    /// Upscales an image to the target dimensions.
    ///
    /// Uses Real-ESRGAN 4x upscaling, then downscales with Lanczos3 if needed.
    /// This produces better quality than direct interpolation.
    ///
    /// # Errors
    ///
    /// Returns an error if the upscaling inference fails.
    pub fn upscale_to_size(
        &mut self,
        image: &DynamicImage,
        target_width: u32,
        target_height: u32,
    ) -> UpscaleResult<DynamicImage> {
        // First, apply 4x Real-ESRGAN upscaling
        let upscaled = self.upscale(image)?;

        // If target is exactly 4x, return as-is
        let upscaled_width = image.width() * UPSCALE_FACTOR;
        let upscaled_height = image.height() * UPSCALE_FACTOR;

        if target_width == upscaled_width && target_height == upscaled_height {
            return Ok(upscaled);
        }

        // Otherwise, resize to target using Lanczos3
        Ok(upscaled.resize_exact(
            target_width,
            target_height,
            image_rs::imageops::FilterType::Lanczos3,
        ))
    }

    /// Deletes the model file from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be deleted.
    pub fn delete_model(&mut self) -> UpscaleResult<()> {
        self.session = None;
        if self.model_path.exists() {
            std::fs::remove_file(&self.model_path).map_err(|e| UpscaleError::Io(e.to_string()))?;
        }
        Ok(())
    }
}

/// Returns the path where the upscale model should be stored.
#[must_use] 
pub fn get_model_path() -> PathBuf {
    paths::get_app_data_dir().map_or_else(
        || PathBuf::from(MODEL_FILENAME),
        |mut p| {
            p.push(MODEL_FILENAME);
            p
        },
    )
}

/// Minimum expected model size (60 MB) to detect failed downloads.
const MIN_MODEL_SIZE_BYTES: u64 = 60_000_000;

/// Checks if the model file exists at the expected location with valid size.
#[must_use] 
pub fn is_model_downloaded() -> bool {
    let path = get_model_path();
    if !path.exists() {
        return false;
    }
    // Also check file size to detect incomplete downloads
    match std::fs::metadata(&path) {
        Ok(meta) => meta.len() >= MIN_MODEL_SIZE_BYTES,
        Err(_) => false,
    }
}

/// Downloads the model from the specified URL.
///
/// Returns the number of bytes downloaded.
///
/// # Errors
///
/// Returns an error if the download fails or the file cannot be written.
pub async fn download_model(
    url: &str,
    mut progress_callback: impl FnMut(f32) + Send,
) -> UpscaleResult<u64> {
    use futures_util::StreamExt;

    // Build client with explicit redirect policy and user agent
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent("IcedLens/0.3.0")
        .build()
        .map_err(|e| UpscaleError::DownloadFailed(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| UpscaleError::DownloadFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(UpscaleError::DownloadFailed(format!(
            "HTTP status: {}",
            response.status()
        )));
    }

    let total_size = response.content_length().unwrap_or(0);

    // Sanity check: if the content length is suspiciously small, something went wrong
    if total_size > 0 && total_size < MIN_MODEL_SIZE_BYTES {
        return Err(UpscaleError::DownloadFailed(format!(
            "Response too small ({total_size} bytes), expected model file (~64 MB). URL may have changed or returned an error page."
        )));
    }

    let model_path = get_model_path();

    // Ensure parent directory exists
    if let Some(parent) = model_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| UpscaleError::Io(e.to_string()))?;
    }

    let mut file =
        std::fs::File::create(&model_path).map_err(|e| UpscaleError::Io(e.to_string()))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| UpscaleError::DownloadFailed(e.to_string()))?;
        std::io::Write::write_all(&mut file, &chunk)
            .map_err(|e| UpscaleError::Io(e.to_string()))?;

        downloaded += chunk.len() as u64;

        if total_size > 0 {
            // Progress percentage - precision loss acceptable for display purposes
            // f64 to f32 truncation is fine for progress display (0.0-1.0 range)
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let progress = (downloaded as f64 / total_size as f64) as f32;
            progress_callback(progress);
        }
    }

    // Final size check
    if downloaded < MIN_MODEL_SIZE_BYTES {
        // Delete the incomplete/invalid file
        let _ = std::fs::remove_file(&model_path);
        return Err(UpscaleError::DownloadFailed(format!(
            "Downloaded file too small ({downloaded} bytes), expected ~64 MB"
        )));
    }

    Ok(downloaded)
}

/// Verifies the model file integrity using BLAKE3 hash.
///
/// # Errors
///
/// Returns an error if the model file is not found, cannot be read,
/// or the checksum does not match.
pub fn verify_checksum(expected_hash: &str) -> UpscaleResult<()> {
    let model_path = get_model_path();
    if !model_path.exists() {
        return Err(UpscaleError::ModelNotFound);
    }

    let file_data = std::fs::read(&model_path).map_err(|e| UpscaleError::Io(e.to_string()))?;
    let actual_hash = blake3::hash(&file_data).to_hex().to_string();

    if actual_hash != expected_hash {
        return Err(UpscaleError::ChecksumMismatch {
            expected: expected_hash.to_string(),
            actual: actual_hash,
        });
    }

    Ok(())
}

/// Computes the BLAKE3 hash of the model file.
///
/// # Errors
///
/// Returns an error if the model file is not found or cannot be read.
pub fn compute_model_hash() -> UpscaleResult<String> {
    let model_path = get_model_path();
    if !model_path.exists() {
        return Err(UpscaleError::ModelNotFound);
    }

    let file_data = std::fs::read(&model_path).map_err(|e| UpscaleError::Io(e.to_string()))?;
    Ok(blake3::hash(&file_data).to_hex().to_string())
}

/// Validates the model by running a test inference.
///
/// Uses a small 64x64 test image as Real-ESRGAN can handle any input size.
///
/// If a cancellation token is provided and triggered, returns `UpscaleError::Cancelled`.
///
/// # Errors
///
/// Returns an error if validation is cancelled or the model fails inference.
pub fn validate_model(
    manager: &mut UpscaleManager,
    cancel_token: Option<&CancellationToken>,
) -> UpscaleResult<()> {
    // Check for cancellation before starting
    if let Some(token) = cancel_token {
        if is_cancelled(token) {
            return Err(UpscaleError::Cancelled);
        }
    }

    // Create a small test image (Real-ESRGAN can handle any size)
    let mut img = image_rs::RgbImage::new(64, 64);
    for pixel in img.pixels_mut() {
        *pixel = image_rs::Rgb([128, 128, 128]); // Gray
    }
    let test_image = DynamicImage::ImageRgb8(img);

    // Check for cancellation before inference (which is atomic and cannot be interrupted)
    if let Some(token) = cancel_token {
        if is_cancelled(token) {
            return Err(UpscaleError::Cancelled);
        }
    }

    // Run inference to validate the model works
    let result = manager.upscale(&test_image)?;

    // Verify output dimensions (should be 4x input)
    if result.width() != 64 * UPSCALE_FACTOR || result.height() != 64 * UPSCALE_FACTOR {
        return Err(UpscaleError::InferenceFailed(format!(
            "Unexpected output size: {}x{}, expected {}x{}",
            result.width(),
            result.height(),
            64 * UPSCALE_FACTOR,
            64 * UPSCALE_FACTOR
        )));
    }

    Ok(())
}

/// Preprocesses an image for Real-ESRGAN inference.
///
/// Converts to NCHW format (batch=1, channels=3, height, width),
/// RGB color order, normalized to 0-1 range.
#[allow(clippy::unnecessary_wraps)] // Result for API consistency with other processing functions
fn preprocess_image(img: &DynamicImage) -> UpscaleResult<Array4<f32>> {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    // Create NCHW tensor (batch=1, channels=3, height, width)
    let mut tensor = Array4::<f32>::zeros((1, 3, height as usize, width as usize));

    for (x, y, pixel) in rgb.enumerate_pixels() {
        let [r, g, b] = pixel.0;
        // Normalize to 0-1 range, RGB order (u8 to f32 is lossless via From)
        tensor[[0, 0, y as usize, x as usize]] = f32::from(r) / 255.0;
        tensor[[0, 1, y as usize, x as usize]] = f32::from(g) / 255.0;
        tensor[[0, 2, y as usize, x as usize]] = f32::from(b) / 255.0;
    }

    Ok(tensor)
}

/// Postprocesses Real-ESRGAN output back to an image.
///
/// Converts from NCHW format (RGB order), denormalizes from 0-1 to 0-255,
/// and clips values to valid range.
fn postprocess_output(outputs: &ort::session::SessionOutputs<'_>) -> UpscaleResult<DynamicImage> {
    // Get the first output tensor
    let (_, output) = outputs
        .iter()
        .next()
        .ok_or_else(|| UpscaleError::PostprocessingFailed("No output tensor".to_string()))?;

    let (shape, data) = output
        .try_extract_tensor::<f32>()
        .map_err(|e: ort::Error| UpscaleError::PostprocessingFailed(e.to_string()))?;

    // Shape is NCHW: [batch, channels, height, width]
    if shape.len() != 4 {
        return Err(UpscaleError::PostprocessingFailed(format!(
            "Expected 4D tensor, got {}D",
            shape.len()
        )));
    }

    // Convert i64 dimensions to usize (validated to be positive by ONNX)
    let height = usize::try_from(shape[2])
        .map_err(|_| UpscaleError::PostprocessingFailed("Invalid tensor height".to_string()))?;
    let width = usize::try_from(shape[3])
        .map_err(|_| UpscaleError::PostprocessingFailed("Invalid tensor width".to_string()))?;
    let channel_size = height * width;

    // Create RGB image (model outputs RGB order)
    let mut pixels = Vec::with_capacity(width * height * 3);

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            // Output is in RGB order, clamp ensures value is in 0-255 range
            // Safe to convert clamped f32 to u8 (clamp guarantees 0.0..=255.0)
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let r = (data[idx] * 255.0).clamp(0.0, 255.0).round() as u8;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let g = (data[channel_size + idx] * 255.0).clamp(0.0, 255.0).round() as u8;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let b = (data[2 * channel_size + idx] * 255.0).clamp(0.0, 255.0).round() as u8;
            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
        }
    }

    // Convert usize dimensions to u32 for image creation
    let width_u32 = u32::try_from(width)
        .map_err(|_| UpscaleError::PostprocessingFailed("Image width too large".to_string()))?;
    let height_u32 = u32::try_from(height)
        .map_err(|_| UpscaleError::PostprocessingFailed("Image height too large".to_string()))?;

    let rgb_image = image_rs::RgbImage::from_raw(width_u32, height_u32, pixels)
        .ok_or_else(|| UpscaleError::PostprocessingFailed("Failed to create image".to_string()))?;

    Ok(DynamicImage::ImageRgb8(rgb_image))
}

/// Thread-safe wrapper for `UpscaleManager`.
pub type SharedUpscaleManager = Arc<Mutex<UpscaleManager>>;

/// Creates a new shared `UpscaleManager` instance.
#[must_use] 
pub fn create_shared_manager() -> SharedUpscaleManager {
    Arc::new(Mutex::new(UpscaleManager::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_path_returns_valid_path() {
        let path = get_model_path();
        assert!(path.to_string_lossy().contains(MODEL_FILENAME));
    }

    #[test]
    fn test_model_status_default() {
        let status = UpscaleModelStatus::default();
        assert_eq!(status, UpscaleModelStatus::NotDownloaded);
    }

    #[test]
    fn test_upscale_error_display() {
        let err = UpscaleError::ModelNotFound;
        assert_eq!(err.to_string(), "Model file not found");

        let err = UpscaleError::Cancelled;
        assert_eq!(err.to_string(), "Operation cancelled");
    }

    #[test]
    fn test_preprocess_image_creates_correct_shape() {
        let img = DynamicImage::new_rgb8(100, 80);
        let tensor = preprocess_image(&img).unwrap();
        assert_eq!(tensor.shape(), &[1, 3, 80, 100]);
    }

    #[test]
    fn test_preprocess_image_normalizes_values() {
        let mut img = image_rs::RgbImage::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = image_rs::Rgb([255, 128, 0]); // R=255, G=128, B=0
        }
        let dynamic = DynamicImage::ImageRgb8(img);

        let tensor = preprocess_image(&dynamic).unwrap();

        // Check normalization - RGB order
        assert!((tensor[[0, 0, 0, 0]] - 1.0).abs() < 0.01); // Red channel (255 -> 1.0)
        assert!((tensor[[0, 1, 0, 0]] - 0.502).abs() < 0.01); // Green channel (128 -> ~0.5)
        assert!(tensor[[0, 2, 0, 0]].abs() < 0.01); // Blue channel (0 -> 0.0)
    }

    #[test]
    fn test_upscale_manager_new() {
        let manager = UpscaleManager::new();
        assert!(!manager.is_session_ready());
    }

    #[test]
    fn test_upscale_factor() {
        assert_eq!(UPSCALE_FACTOR, 4);
    }
}
