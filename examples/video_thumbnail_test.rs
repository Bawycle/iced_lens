// SPDX-License-Identifier: MPL-2.0
//! Test video thumbnail extraction with a real video file.

use iced_lens::media::video;
use std::env;
use std::error::Error;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    // Get video path from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_video.mp4>", args[0]);
        eprintln!("\nPlease provide a path to a video file (MP4, MKV, AVI, etc.)");
        std::process::exit(1);
    }

    let video_path = &args[1];
    println!("🎬 Testing thumbnail extraction from: {video_path}");

    // Measure extraction time
    let start = Instant::now();
    let result = video::extract_thumbnail(video_path);
    let duration = start.elapsed();

    match result {
        Ok(image_data) => {
            println!("✅ Thumbnail extracted successfully!");
            println!("   Dimensions: {}x{}", image_data.width, image_data.height);
            println!("   Time taken: {duration:?}");

            // Check if extraction time is acceptable (< 500ms target for 1080p)
            if duration.as_millis() < 500 {
                println!("   ⚡ Performance: EXCELLENT (< 500ms)");
            } else if duration.as_millis() < 1000 {
                println!("   ✓ Performance: GOOD (< 1s)");
            } else {
                println!("   ⚠ Performance: SLOW (> 1s)");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to extract thumbnail: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}
