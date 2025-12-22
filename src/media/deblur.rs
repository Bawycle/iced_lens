// SPDX-License-Identifier: MPL-2.0
//! AI-powered image deblurring using NAFNet ONNX model.
//!
//! This module provides functionality for:
//! - Downloading the NAFNet ONNX model from a configurable URL
//! - Verifying model integrity with BLAKE3 checksum
//! - Running inference to deblur images

use crate::app::paths;

/// Filename for the downloaded deblur model in the data directory.
const MODEL_FILENAME: &str = "nafnet-deblur.onnx";
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

/// Result type for deblur operations.
pub type DeblurResult<T> = Result<T, DeblurError>;

/// Errors that can occur during deblurring operations.
#[derive(Debug, Clone)]
pub enum DeblurError {
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

impl std::fmt::Display for DeblurError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeblurError::ModelNotFound => write!(f, "Model file not found"),
            DeblurError::DownloadFailed(msg) => write!(f, "Download failed: {msg}"),
            DeblurError::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "Checksum mismatch: expected {expected}, got {actual}"
                )
            }
            DeblurError::InferenceFailed(msg) => write!(f, "Inference failed: {msg}"),
            DeblurError::PreprocessingFailed(msg) => write!(f, "Preprocessing failed: {msg}"),
            DeblurError::PostprocessingFailed(msg) => write!(f, "Postprocessing failed: {msg}"),
            DeblurError::Cancelled => write!(f, "Operation cancelled"),
            DeblurError::Io(msg) => write!(f, "IO error: {msg}"),
            DeblurError::SessionNotInitialized => write!(f, "ONNX session not initialized"),
        }
    }
}

impl std::error::Error for DeblurError {}

/// Status of the deblur model.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ModelStatus {
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

/// Manager for the NAFNet deblurring model.
///
/// Handles model lifecycle: download, validation, and inference.
pub struct DeblurManager {
    model_path: PathBuf,
    session: Option<Session>,
}

impl Default for DeblurManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DeblurManager {
    /// Creates a new DeblurManager instance.
    pub fn new() -> Self {
        let model_path = get_model_path();
        Self {
            model_path,
            session: None,
        }
    }

    /// Returns the path where the model is/will be stored.
    pub fn model_path(&self) -> &PathBuf {
        &self.model_path
    }

    /// Checks if the model file exists on disk.
    pub fn is_model_downloaded(&self) -> bool {
        self.model_path.exists()
    }

    /// Loads the ONNX session from the model file.
    ///
    /// Must be called after the model is downloaded and verified.
    /// If a cancellation token is provided and triggered, returns `DeblurError::Cancelled`.
    pub fn load_session(&mut self, cancel_token: Option<&CancellationToken>) -> DeblurResult<()> {
        // Check for cancellation before loading
        if let Some(token) = cancel_token {
            if is_cancelled(token) {
                return Err(DeblurError::Cancelled);
            }
        }

        if !self.model_path.exists() {
            return Err(DeblurError::ModelNotFound);
        }

        let session = Session::builder()
            .map_err(|e| DeblurError::InferenceFailed(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| DeblurError::InferenceFailed(e.to_string()))?
            .commit_from_file(&self.model_path)
            .map_err(|e| DeblurError::InferenceFailed(e.to_string()))?;

        self.session = Some(session);
        Ok(())
    }

    /// Checks if the ONNX session is loaded and ready.
    pub fn is_session_ready(&self) -> bool {
        self.session.is_some()
    }

    /// Runs deblurring inference on an image.
    ///
    /// Returns the deblurred image. Small images are automatically padded
    /// to meet the minimum dimension requirement, then cropped back.
    pub fn deblur(&mut self, image: &DynamicImage) -> DeblurResult<DynamicImage> {
        let session = self
            .session
            .as_mut()
            .ok_or(DeblurError::SessionNotInitialized)?;

        // Store original dimensions for cropping after inference
        let original_width = image.width();
        let original_height = image.height();

        // Preprocess: DynamicImage -> NCHW tensor (RGB, normalized 0-1, padded if needed)
        let input_tensor = preprocess_image(image)?;

        // Ensure standard layout for ONNX Runtime
        let input_tensor = input_tensor.as_standard_layout().into_owned();

        // Get input name from model (NAFNet uses 'lq' for low-quality input)
        let input_name = session
            .inputs
            .first()
            .map_or_else(|| "lq".to_string(), |i| i.name.clone());

        // Create tensor reference for inference
        let input_ref = ort::value::TensorRef::from_array_view(&input_tensor)
            .map_err(|e| DeblurError::InferenceFailed(e.to_string()))?;

        let outputs = session
            .run(ort::inputs![input_name.as_str() => input_ref])
            .map_err(|e| DeblurError::InferenceFailed(e.to_string()))?;

        // Postprocess: NCHW tensor -> DynamicImage (cropped to original size)
        postprocess_output(&outputs, original_width, original_height)
    }

    /// Deletes the model file from disk.
    pub fn delete_model(&mut self) -> DeblurResult<()> {
        self.session = None;
        if self.model_path.exists() {
            std::fs::remove_file(&self.model_path).map_err(|e| DeblurError::Io(e.to_string()))?;
        }
        Ok(())
    }
}

/// Returns the path where the deblur model should be stored.
pub fn get_model_path() -> PathBuf {
    paths::get_app_data_dir().map_or_else(
        || PathBuf::from(MODEL_FILENAME),
        |mut p| {
            p.push(MODEL_FILENAME);
            p
        },
    )
}

/// Minimum expected model size (80 MB) to detect failed downloads.
const MIN_MODEL_SIZE_BYTES: u64 = 80_000_000;

/// Checks if the model file exists at the expected location with valid size.
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
pub async fn download_model(
    url: &str,
    mut progress_callback: impl FnMut(f32) + Send,
) -> DeblurResult<u64> {
    use futures_util::StreamExt;

    // Build client with explicit redirect policy and user agent
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent("IcedLens/0.3.0")
        .build()
        .map_err(|e| DeblurError::DownloadFailed(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| DeblurError::DownloadFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(DeblurError::DownloadFailed(format!(
            "HTTP status: {}",
            response.status()
        )));
    }

    let total_size = response.content_length().unwrap_or(0);

    // Sanity check: if the content length is suspiciously small, something went wrong
    if total_size > 0 && total_size < MIN_MODEL_SIZE_BYTES {
        return Err(DeblurError::DownloadFailed(format!(
            "Response too small ({total_size} bytes), expected model file (~92 MB). URL may have changed or returned an error page."
        )));
    }

    let model_path = get_model_path();

    // Ensure parent directory exists
    if let Some(parent) = model_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| DeblurError::Io(e.to_string()))?;
    }

    let mut file =
        std::fs::File::create(&model_path).map_err(|e| DeblurError::Io(e.to_string()))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| DeblurError::DownloadFailed(e.to_string()))?;
        std::io::Write::write_all(&mut file, &chunk).map_err(|e| DeblurError::Io(e.to_string()))?;

        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = downloaded as f32 / total_size as f32;
            progress_callback(progress);
        }
    }

    // Final size check
    if downloaded < MIN_MODEL_SIZE_BYTES {
        // Delete the incomplete/invalid file
        let _ = std::fs::remove_file(&model_path);
        return Err(DeblurError::DownloadFailed(format!(
            "Downloaded file too small ({downloaded} bytes), expected ~92 MB"
        )));
    }

    Ok(downloaded)
}

/// Verifies the model file integrity using BLAKE3 hash.
pub fn verify_checksum(expected_hash: &str) -> DeblurResult<()> {
    let model_path = get_model_path();
    if !model_path.exists() {
        return Err(DeblurError::ModelNotFound);
    }

    let file_data = std::fs::read(&model_path).map_err(|e| DeblurError::Io(e.to_string()))?;
    let actual_hash = blake3::hash(&file_data).to_hex().to_string();

    if actual_hash != expected_hash {
        return Err(DeblurError::ChecksumMismatch {
            expected: expected_hash.to_string(),
            actual: actual_hash,
        });
    }

    Ok(())
}

/// Computes the BLAKE3 hash of the model file.
pub fn compute_model_hash() -> DeblurResult<String> {
    let model_path = get_model_path();
    if !model_path.exists() {
        return Err(DeblurError::ModelNotFound);
    }

    let file_data = std::fs::read(&model_path).map_err(|e| DeblurError::Io(e.to_string()))?;
    Ok(blake3::hash(&file_data).to_hex().to_string())
}

/// Validates the model by running a test inference.
///
/// Uses a 1024x1024 test image as NAFNet's architecture requires large
/// spatial dimensions for its many encoder stages (each stage halves dimensions).
/// The OpenCV NAFNet model requires minimum ~1024x1024 input to avoid
/// zero-sized internal tensors during inference.
///
/// If a cancellation token is provided and triggered, returns `DeblurError::Cancelled`.
pub fn validate_model(
    manager: &mut DeblurManager,
    cancel_token: Option<&CancellationToken>,
) -> DeblurResult<()> {
    // Check for cancellation before starting
    if let Some(token) = cancel_token {
        if is_cancelled(token) {
            return Err(DeblurError::Cancelled);
        }
    }

    // Create a test image with sufficient size for NAFNet's encoder stages.
    // NAFNet requires minimum 1024x1024 input due to its many downsampling stages.
    // Use a gray image (not black) to avoid potential issues with all-zero inputs.
    let mut img = image_rs::RgbImage::new(1024, 1024);
    for pixel in img.pixels_mut() {
        *pixel = image_rs::Rgb([128, 128, 128]); // Gray
    }
    let test_image = DynamicImage::ImageRgb8(img);

    // Check for cancellation before inference (which is atomic and cannot be interrupted)
    if let Some(token) = cancel_token {
        if is_cancelled(token) {
            return Err(DeblurError::Cancelled);
        }
    }

    // Run inference to validate the model works
    let _result = manager.deblur(&test_image)?;

    Ok(())
}

/// Minimum dimension for NAFNet inference.
/// NAFNet's encoder stages halve dimensions multiple times, requiring sufficient
/// spatial resolution to avoid zero-sized internal tensors.
const MIN_DIMENSION: u32 = 1024;

/// Preprocesses an image for NAFNet inference.
///
/// Converts to NCHW format (batch=1, channels=3, height, width),
/// RGB color order, normalized to 0-1 range.
///
/// If the image is smaller than MIN_DIMENSION, it will be padded with
/// edge reflection to meet the minimum size requirement.
fn preprocess_image(img: &DynamicImage) -> DeblurResult<Array4<f32>> {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    // Pad image if too small for NAFNet's encoder stages
    let (padded_rgb, padded_width, padded_height) =
        if width < MIN_DIMENSION || height < MIN_DIMENSION {
            let new_width = width.max(MIN_DIMENSION);
            let new_height = height.max(MIN_DIMENSION);
            let padded = pad_image_reflect(&rgb, new_width, new_height);
            (padded, new_width, new_height)
        } else {
            (rgb, width, height)
        };

    // Create NCHW tensor (batch=1, channels=3, height, width)
    let mut tensor = Array4::<f32>::zeros((1, 3, padded_height as usize, padded_width as usize));

    for (x, y, pixel) in padded_rgb.enumerate_pixels() {
        let [r, g, b] = pixel.0;
        // Normalize to 0-1 range, RGB order
        tensor[[0, 0, y as usize, x as usize]] = r as f32 / 255.0;
        tensor[[0, 1, y as usize, x as usize]] = g as f32 / 255.0;
        tensor[[0, 2, y as usize, x as usize]] = b as f32 / 255.0;
    }

    Ok(tensor)
}

/// Pads an image using edge reflection to reach target dimensions.
fn pad_image_reflect(
    img: &image_rs::RgbImage,
    target_width: u32,
    target_height: u32,
) -> image_rs::RgbImage {
    let (src_width, src_height) = img.dimensions();
    let mut padded = image_rs::RgbImage::new(target_width, target_height);

    for y in 0..target_height {
        for x in 0..target_width {
            // Reflect coordinates for padding
            let src_x = if x < src_width {
                x
            } else {
                // Mirror reflection
                let overflow = x - src_width;
                if overflow < src_width {
                    src_width - 1 - overflow
                } else {
                    0
                }
            };
            let src_y = if y < src_height {
                y
            } else {
                let overflow = y - src_height;
                if overflow < src_height {
                    src_height - 1 - overflow
                } else {
                    0
                }
            };
            padded.put_pixel(x, y, *img.get_pixel(src_x, src_y));
        }
    }

    padded
}

/// Postprocesses NAFNet output back to an image.
///
/// Converts from NCHW format (RGB order), denormalizes from 0-1 to 0-255,
/// clips values, and optionally crops to original dimensions if padding was applied.
fn postprocess_output(
    outputs: &ort::session::SessionOutputs<'_>,
    original_width: u32,
    original_height: u32,
) -> DeblurResult<DynamicImage> {
    // Get the first output tensor
    let (_, output) = outputs
        .iter()
        .next()
        .ok_or_else(|| DeblurError::PostprocessingFailed("No output tensor".to_string()))?;

    let (shape, data) = output
        .try_extract_tensor::<f32>()
        .map_err(|e: ort::Error| DeblurError::PostprocessingFailed(e.to_string()))?;

    // Shape is NCHW: [batch, channels, height, width]
    if shape.len() != 4 {
        return Err(DeblurError::PostprocessingFailed(format!(
            "Expected 4D tensor, got {}D",
            shape.len()
        )));
    }

    let height = shape[2] as usize;
    let width = shape[3] as usize;
    let channel_size = height * width;

    // Create RGB image (model outputs RGB order)
    let mut pixels = Vec::with_capacity(width * height * 3);

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            // Output is in RGB order
            let r = (data[idx] * 255.0).clamp(0.0, 255.0) as u8;
            let g = (data[channel_size + idx] * 255.0).clamp(0.0, 255.0) as u8;
            let b = (data[2 * channel_size + idx] * 255.0).clamp(0.0, 255.0) as u8;
            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
        }
    }

    let rgb_image = image_rs::RgbImage::from_raw(width as u32, height as u32, pixels)
        .ok_or_else(|| DeblurError::PostprocessingFailed("Failed to create image".to_string()))?;

    let result = DynamicImage::ImageRgb8(rgb_image);

    // Crop back to original dimensions if padding was applied
    if width as u32 != original_width || height as u32 != original_height {
        Ok(result.crop_imm(0, 0, original_width, original_height))
    } else {
        Ok(result)
    }
}

/// Thread-safe wrapper for DeblurManager.
pub type SharedDeblurManager = Arc<Mutex<DeblurManager>>;

/// Creates a new shared DeblurManager instance.
pub fn create_shared_manager() -> SharedDeblurManager {
    Arc::new(Mutex::new(DeblurManager::new()))
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
        let status = ModelStatus::default();
        assert_eq!(status, ModelStatus::NotDownloaded);
    }

    #[test]
    fn test_deblur_error_display() {
        let err = DeblurError::ModelNotFound;
        assert_eq!(err.to_string(), "Model file not found");

        let err = DeblurError::Cancelled;
        assert_eq!(err.to_string(), "Operation cancelled");
    }

    #[test]
    fn test_preprocess_image_creates_correct_shape() {
        // Large image (no padding needed)
        let img = DynamicImage::new_rgb8(1920, 1080);
        let tensor = preprocess_image(&img).unwrap();
        assert_eq!(tensor.shape(), &[1, 3, 1080, 1920]);

        // Small image (will be padded to MIN_DIMENSION)
        let small_img = DynamicImage::new_rgb8(32, 24);
        let tensor = preprocess_image(&small_img).unwrap();
        // Both dimensions are padded to MIN_DIMENSION (1024)
        assert_eq!(tensor.shape(), &[1, 3, 1024, 1024]);
    }

    #[test]
    fn test_preprocess_image_normalizes_values() {
        // Create a test image with known RGB values (must be >= MIN_DIMENSION to avoid padding)
        let mut img = image_rs::RgbImage::new(1024, 1024);
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
    fn test_deblur_manager_new() {
        let manager = DeblurManager::new();
        assert!(!manager.is_session_ready());
    }
}
