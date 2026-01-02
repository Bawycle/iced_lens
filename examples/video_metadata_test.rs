// SPDX-License-Identifier: MPL-2.0
//! Test video metadata extraction with a real video file.

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use iced_lens::media::video;
use std::env;
use std::error::Error;

#[allow(clippy::unnecessary_wraps)]
fn main() -> Result<(), Box<dyn Error>> {
    // Get video path from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_video.mp4>", args[0]);
        eprintln!("\nPlease provide a path to a video file (MP4, MKV, AVI, etc.)");
        std::process::exit(1);
    }

    let video_path = &args[1];
    println!("🎬 Extracting metadata from: {video_path}");

    // Extract metadata
    let result = video::extract_video_metadata(video_path);

    match result {
        Ok(metadata) => {
            println!("✅ Metadata extracted successfully!");
            println!("\n📐 Video Information:");
            println!("   Dimensions: {}x{}", metadata.width, metadata.height);
            println!("   Duration:   {:.2}s", metadata.duration_secs);
            println!("   Frame rate: {:.2} fps", metadata.fps);
            println!(
                "   Audio:      {}",
                if metadata.has_audio {
                    "Yes 🔊"
                } else {
                    "No 🔇"
                }
            );

            // Calculate additional info
            let total_frames = (metadata.duration_secs * metadata.fps) as u64;
            let resolution_class = match (metadata.width, metadata.height) {
                (w, h) if w >= 3840 && h >= 2160 => "4K UHD",
                (w, h) if w >= 2560 && h >= 1440 => "2K QHD",
                (w, h) if w >= 1920 && h >= 1080 => "1080p Full HD",
                (w, h) if w >= 1280 && h >= 720 => "720p HD",
                (w, h) if w >= 854 && h >= 480 => "480p SD",
                _ => "Low Resolution",
            };

            println!("\n📊 Calculated:");
            println!("   Resolution: {resolution_class}");
            println!("   Total frames: ~{total_frames}");
            println!(
                "   Aspect ratio: {:.2}",
                f64::from(metadata.width) / f64::from(metadata.height)
            );
        }
        Err(e) => {
            eprintln!("❌ Failed to extract metadata: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}
