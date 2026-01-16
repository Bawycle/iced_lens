// SPDX-License-Identifier: MPL-2.0
//! ONNX Runtime adapters implementing the [`AIProcessor`] port trait.
//!
//! This module provides AI image processing adapters:
//!
//! - [`OnnxDeblurProcessor`]: `NAFNet`-based image deblurring
//! - [`OnnxUpscaleProcessor`]: `Real-ESRGAN`-based 4x upscaling
//!
//! # Design Notes
//!
//! - These adapters wrap the existing `DeblurManager` and `UpscaleManager`
//! - They convert between domain types ([`RawImage`]) and image crate types
//! - The underlying managers handle model loading and ONNX inference
//!
//! [`AIProcessor`]: crate::application::port::AIProcessor
//! [`RawImage`]: crate::domain::media::RawImage

mod deblur;
mod upscale;

pub use deblur::OnnxDeblurProcessor;
pub use upscale::OnnxUpscaleProcessor;
