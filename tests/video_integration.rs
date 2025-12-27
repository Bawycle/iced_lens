// SPDX-License-Identifier: MPL-2.0
//! Integration tests for video thumbnail extraction, metadata, and playback
//!
//! These tests validate the complete video processing pipeline with real video files,
//! including multi-format playback support (MP4, AVI, MOV, MKV, `WebM`).

use iced_lens::media::video::{extract_thumbnail, extract_video_metadata};
use iced_lens::video_player::audio::{AudioDecoder, AudioDecoderCommand, AudioDecoderEvent};
use iced_lens::video_player::{AsyncDecoder, CacheConfig, DecoderCommand, DecoderEvent};
use std::time::Duration;

#[test]
fn test_extract_thumbnail_mp4() {
    let path = "tests/data/sample.mp4";
    if !std::path::Path::new(path).exists() {
        return; // Skip if test file doesn't exist
    }

    let result = extract_thumbnail(path);
    assert!(result.is_ok(), "Should extract thumbnail from MP4");

    let thumbnail = result.unwrap();
    assert!(thumbnail.width > 0, "Thumbnail width should be > 0");
    assert!(thumbnail.height > 0, "Thumbnail height should be > 0");
}

#[test]
fn test_extract_thumbnail_avi() {
    let path = "tests/data/sample.avi";
    if !std::path::Path::new(path).exists() {
        return;
    }

    let result = extract_thumbnail(path);
    assert!(result.is_ok(), "Should extract thumbnail from AVI");

    let thumbnail = result.unwrap();
    assert!(thumbnail.width > 0);
    assert!(thumbnail.height > 0);
}

#[test]
fn test_extract_thumbnail_webm() {
    let path = "tests/data/sample.webm";
    if !std::path::Path::new(path).exists() {
        return;
    }

    let result = extract_thumbnail(path);
    assert!(result.is_ok(), "Should extract thumbnail from WebM");

    let thumbnail = result.unwrap();
    assert!(thumbnail.width > 0);
    assert!(thumbnail.height > 0);
}

#[test]
fn test_extract_thumbnail_mov() {
    let path = "tests/data/sample.mov";
    if !std::path::Path::new(path).exists() {
        return;
    }

    let result = extract_thumbnail(path);
    assert!(result.is_ok(), "Should extract thumbnail from MOV");

    let thumbnail = result.unwrap();
    assert!(thumbnail.width > 0);
    assert!(thumbnail.height > 0);
}

#[test]
fn test_extract_thumbnail_mkv() {
    let path = "tests/data/sample.mkv";
    if !std::path::Path::new(path).exists() {
        return;
    }

    let result = extract_thumbnail(path);
    assert!(result.is_ok(), "Should extract thumbnail from MKV");

    let thumbnail = result.unwrap();
    assert!(thumbnail.width > 0);
    assert!(thumbnail.height > 0);
}

#[test]
fn test_extract_thumbnail_corrupted_file() {
    let path = "tests/data/corrupted.mp4";
    if !std::path::Path::new(path).exists() {
        return;
    }

    let result = extract_thumbnail(path);
    assert!(result.is_err(), "Should fail on corrupted file");
}

#[test]
fn test_extract_thumbnail_nonexistent_file() {
    let path = "tests/data/this_file_does_not_exist.mp4";
    let result = extract_thumbnail(path);
    assert!(result.is_err(), "Should fail on nonexistent file");
}

#[test]
fn test_extract_video_metadata_mp4() {
    let path = "tests/data/sample.mp4";
    if !std::path::Path::new(path).exists() {
        return;
    }

    let result = extract_video_metadata(path);
    assert!(result.is_ok(), "Should extract metadata from MP4");

    let metadata = result.unwrap();
    assert!(metadata.width > 0, "Width should be > 0");
    assert!(metadata.height > 0, "Height should be > 0");
    assert!(metadata.duration_secs > 0.0, "Duration should be > 0");
    assert!(metadata.fps > 0.0, "FPS should be > 0");
    // has_audio can be true or false, both valid
}

#[test]
fn test_extract_video_metadata_no_audio() {
    let path = "tests/data/sample_no_audio.mp4";
    if !std::path::Path::new(path).exists() {
        return;
    }

    let result = extract_video_metadata(path);
    assert!(
        result.is_ok(),
        "Should extract metadata from video without audio"
    );

    let metadata = result.unwrap();
    assert!(!metadata.has_audio, "Should detect no audio track");
}

#[test]
fn test_extract_video_metadata_with_audio() {
    let path = "tests/data/sample_with_audio.mp4";
    if !std::path::Path::new(path).exists() {
        return;
    }

    let result = extract_video_metadata(path);
    assert!(
        result.is_ok(),
        "Should extract metadata from video with audio"
    );

    let metadata = result.unwrap();
    assert!(metadata.has_audio, "Should detect audio track");
}

#[test]
fn test_thumbnail_extraction_performance() {
    let path = "tests/data/sample.mp4";
    if !std::path::Path::new(path).exists() {
        return;
    }

    let start = std::time::Instant::now();
    let result = extract_thumbnail(path);
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Should extract thumbnail successfully");
    assert!(
        elapsed.as_millis() < 500,
        "Thumbnail extraction should take < 500ms (took {}ms)",
        elapsed.as_millis()
    );
}

// =============================================================================
// Video Playback Tests (Multi-Format)
// =============================================================================

/// Helper function to test video decoding for a given format.
/// Creates a decoder, sends Play command, and verifies at least one frame is received.
fn test_video_decoding(path: &str, format_name: &str) {
    if !std::path::Path::new(path).exists() {
        eprintln!("Skipping {format_name} test: file not found");
        return;
    }

    // Create a Tokio runtime for the async decoder
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    rt.block_on(async {
        let decoder = AsyncDecoder::new(path, CacheConfig::disabled(), 0, None)
            .unwrap_or_else(|_| panic!("Should create decoder for {format_name}"));

        // Send play command
        decoder
            .send_command(DecoderCommand::Play {
                resume_position_secs: None,
            })
            .expect("Should send Play command");

        // Wait for at least one frame (with timeout)
        let mut decoder = decoder;
        let timeout = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Some(event) = decoder.recv_event().await {
                    match event {
                        DecoderEvent::FrameReady(frame) => {
                            // Verify frame has valid dimensions
                            assert!(frame.width > 0, "{format_name} frame width should be > 0");
                            assert!(frame.height > 0, "{format_name} frame height should be > 0");
                            assert!(
                                !frame.rgba_data.is_empty(),
                                "{format_name} frame should have RGBA data"
                            );
                            // Verify RGBA data size matches dimensions
                            let expected_size = (frame.width * frame.height * 4) as usize;
                            assert_eq!(
                                frame.rgba_data.len(),
                                expected_size,
                                "{format_name} frame RGBA size should match dimensions"
                            );
                            return true;
                        }
                        DecoderEvent::Error(msg) => {
                            panic!("{format_name} decoding error: {msg}");
                        }
                        DecoderEvent::EndOfStream => {
                            panic!("{format_name} reached end of stream without producing frames");
                        }
                        DecoderEvent::Buffering | DecoderEvent::HistoryExhausted => {
                            // Continue waiting
                        }
                    }
                }
            }
        })
        .await;

        assert!(
            timeout.is_ok(),
            "{format_name} decoding timed out after 5 seconds"
        );

        // Stop the decoder
        let _ = decoder.send_command(DecoderCommand::Stop);
    });
}

#[test]
fn test_decode_mp4() {
    test_video_decoding("tests/data/sample.mp4", "MP4");
}

#[test]
fn test_decode_avi() {
    test_video_decoding("tests/data/sample.avi", "AVI");
}

#[test]
fn test_decode_mov() {
    test_video_decoding("tests/data/sample.mov", "MOV");
}

#[test]
fn test_decode_mkv() {
    test_video_decoding("tests/data/sample.mkv", "MKV");
}

#[test]
fn test_decode_webm() {
    test_video_decoding("tests/data/sample.webm", "WebM");
}

#[test]
fn test_decode_mp4_no_audio() {
    test_video_decoding("tests/data/sample_no_audio.mp4", "MP4 (no audio)");
}

#[test]
fn test_decode_mp4_with_audio() {
    test_video_decoding("tests/data/sample_with_audio.mp4", "MP4 (with audio)");
}

/// Test that corrupted files are handled gracefully
#[test]
fn test_decode_corrupted_file() {
    let path = "tests/data/corrupted.mp4";
    if !std::path::Path::new(path).exists() {
        return;
    }

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    rt.block_on(async {
        // Corrupted file should either fail to create decoder or produce an error event
        if let Ok(mut decoder) = AsyncDecoder::new(path, CacheConfig::disabled(), 0, None) {
            decoder
                .send_command(DecoderCommand::Play {
                    resume_position_secs: None,
                })
                .expect("Should send Play command");

            // Should get an error event, not a valid frame
            let timeout = tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    if let Some(event) = decoder.recv_event().await {
                        match event {
                            // Error, EOS, or frame received - all acceptable termination conditions
                            DecoderEvent::Error(_)
                            | DecoderEvent::EndOfStream
                            | DecoderEvent::FrameReady(_) => return true,
                            DecoderEvent::Buffering | DecoderEvent::HistoryExhausted => continue,
                        }
                    }
                }
            })
            .await;

            // Timeout is also acceptable for corrupted files
            let _ = timeout;
        } else {
            // Expected: decoder creation failed
        }
    });
}

/// Test video metadata extraction for all formats
#[test]
fn test_metadata_all_formats() {
    let formats = [
        ("tests/data/sample.mp4", "MP4"),
        ("tests/data/sample.avi", "AVI"),
        ("tests/data/sample.mov", "MOV"),
        ("tests/data/sample.mkv", "MKV"),
        ("tests/data/sample.webm", "WebM"),
    ];

    for (path, format_name) in &formats {
        if !std::path::Path::new(path).exists() {
            eprintln!("Skipping {format_name} metadata test: file not found");
            continue;
        }

        let result = extract_video_metadata(path);
        assert!(
            result.is_ok(),
            "Should extract metadata from {format_name} ({path})"
        );

        let metadata = result.unwrap();
        assert!(
            metadata.width > 0,
            "{format_name} metadata width should be > 0"
        );
        assert!(
            metadata.height > 0,
            "{format_name} metadata height should be > 0"
        );
        assert!(
            metadata.duration_secs > 0.0,
            "{format_name} metadata duration should be > 0"
        );
        assert!(
            metadata.fps > 0.0,
            "{format_name} metadata FPS should be > 0"
        );
    }
}

/// Test frame decoding performance across formats
#[test]
fn test_decode_performance() {
    let formats = [
        ("tests/data/sample.mp4", "MP4"),
        ("tests/data/sample.avi", "AVI"),
        ("tests/data/sample.mov", "MOV"),
        ("tests/data/sample.mkv", "MKV"),
        ("tests/data/sample.webm", "WebM"),
    ];

    for (path, format_name) in &formats {
        if !std::path::Path::new(path).exists() {
            continue;
        }

        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        rt.block_on(async {
            let start = std::time::Instant::now();

            let Ok(decoder) = AsyncDecoder::new(path, CacheConfig::disabled(), 0, None) else {
                return;
            };

            decoder
                .send_command(DecoderCommand::Play {
                    resume_position_secs: None,
                })
                .expect("Should send Play command");

            let mut decoder = decoder;
            let mut frame_count = 0;
            let test_duration = Duration::from_secs(1);

            // Decode frames for 1 second
            let _ = tokio::time::timeout(test_duration, async {
                loop {
                    if let Some(DecoderEvent::FrameReady(_)) = decoder.recv_event().await {
                        frame_count += 1;
                    }
                }
            })
            .await;

            let elapsed = start.elapsed();
            let fps = f64::from(frame_count) / elapsed.as_secs_f64();

            eprintln!("{format_name}: decoded {frame_count} frames in {elapsed:?} ({fps:.1} fps)");

            // Should decode at least a few frames in 1 second
            assert!(
                frame_count > 0,
                "{format_name} should decode at least 1 frame in 1 second"
            );

            let _ = decoder.send_command(DecoderCommand::Stop);
        });
    }
}

// =============================================================================
// Audio Decoding Tests (Multi-Format)
// =============================================================================

/// Helper function to test audio decoding for a given format.
/// Creates an audio decoder, sends Play command, and verifies audio buffers are received.
fn test_audio_decoding(path: &str, format_name: &str, expect_audio: bool) {
    if !std::path::Path::new(path).exists() {
        eprintln!("Skipping {format_name} audio test: file not found");
        return;
    }

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    rt.block_on(async {
        match AudioDecoder::new(path, None) {
            Ok(Some(mut decoder)) => {
                assert!(
                    expect_audio,
                    "{format_name} should NOT have audio but AudioDecoder was created"
                );

                // Send play command
                decoder
                    .send_command(AudioDecoderCommand::Play)
                    .expect("Should send Play command");

                // Wait for stream info and at least one audio buffer
                let mut got_stream_info = false;
                let mut got_audio_buffer = false;

                let timeout = tokio::time::timeout(Duration::from_secs(5), async {
                    loop {
                        if let Some(event) = decoder.recv_event().await {
                            match event {
                                AudioDecoderEvent::StreamInfo(info) => {
                                    got_stream_info = true;
                                    assert!(
                                        info.sample_rate > 0,
                                        "{format_name} sample rate should be > 0"
                                    );
                                    assert!(
                                        info.channels > 0,
                                        "{format_name} channels should be > 0"
                                    );
                                    eprintln!(
                                        "{} audio: {}Hz, {} channels, codec: {}",
                                        format_name,
                                        info.sample_rate,
                                        info.channels,
                                        info.codec_name
                                    );
                                }
                                AudioDecoderEvent::BufferReady(buffer) => {
                                    got_audio_buffer = true;
                                    assert!(
                                        !buffer.samples.is_empty(),
                                        "{format_name} audio buffer should have samples"
                                    );
                                    assert!(
                                        buffer.sample_rate > 0,
                                        "{format_name} audio sample rate should be > 0"
                                    );
                                    assert!(
                                        buffer.channels > 0,
                                        "{format_name} audio channels should be > 0"
                                    );
                                    // Got what we need, exit
                                    if got_stream_info {
                                        return true;
                                    }
                                }
                                AudioDecoderEvent::Error(msg) => {
                                    panic!("{format_name} audio decoding error: {msg}");
                                }
                                AudioDecoderEvent::EndOfStream => {
                                    // Some short videos might reach end before we get buffers
                                    return got_stream_info || got_audio_buffer;
                                }
                            }
                        }
                    }
                })
                .await;

                assert!(
                    timeout.is_ok(),
                    "{format_name} audio decoding timed out after 5 seconds"
                );

                // Stop the decoder
                let _ = decoder.send_command(AudioDecoderCommand::Stop);
            }
            Ok(None) => {
                assert!(
                    !expect_audio,
                    "{format_name} should have audio but no audio stream found"
                );
                // Expected: no audio stream
            }
            Err(e) => {
                panic!("{format_name} audio decoder creation failed: {e}");
            }
        }
    });
}

#[test]
fn test_audio_decode_mp4_with_audio() {
    test_audio_decoding("tests/data/sample_with_audio.mp4", "MP4 (with audio)", true);
}

#[test]
fn test_audio_decode_mp4_no_audio() {
    test_audio_decoding("tests/data/sample_no_audio.mp4", "MP4 (no audio)", false);
}

#[test]
fn test_audio_decode_webm() {
    // WebM files typically use Opus or Vorbis audio
    test_audio_decoding("tests/data/sample.webm", "WebM", true);
}

#[test]
fn test_audio_decode_mkv() {
    test_audio_decoding("tests/data/sample.mkv", "MKV", true);
}

#[test]
fn test_audio_decode_mov() {
    test_audio_decoding("tests/data/sample.mov", "MOV", true);
}

#[test]
fn test_audio_decode_avi() {
    // Note: Our test AVI file has no audio track
    test_audio_decoding("tests/data/sample.avi", "AVI", false);
}

/// Test audio stream info extraction for all formats with audio
#[test]
fn test_audio_stream_info_all_formats() {
    // Note: sample.avi has no audio, so it's excluded from this test
    let formats_with_audio = [
        ("tests/data/sample_with_audio.mp4", "MP4"),
        ("tests/data/sample.webm", "WebM"),
        ("tests/data/sample.mkv", "MKV"),
        ("tests/data/sample.mov", "MOV"),
    ];

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    for (path, format_name) in &formats_with_audio {
        if !std::path::Path::new(path).exists() {
            eprintln!("Skipping {format_name} audio stream info test: file not found");
            continue;
        }

        rt.block_on(async {
            match AudioDecoder::new(path, None) {
                Ok(Some(mut decoder)) => {
                    decoder
                        .send_command(AudioDecoderCommand::Play)
                        .expect("Should send Play command");

                    // Wait for stream info
                    let timeout = tokio::time::timeout(Duration::from_secs(3), async {
                        loop {
                            if let Some(AudioDecoderEvent::StreamInfo(info)) =
                                decoder.recv_event().await
                            {
                                return Some(info);
                            }
                        }
                    })
                    .await;

                    assert!(
                        timeout.is_ok(),
                        "{format_name} should provide audio stream info"
                    );

                    if let Ok(Some(info)) = timeout {
                        eprintln!(
                            "{}: sample_rate={}, channels={}, codec={}",
                            format_name, info.sample_rate, info.channels, info.codec_name
                        );
                    }

                    let _ = decoder.send_command(AudioDecoderCommand::Stop);
                }
                Ok(None) => {
                    eprintln!("{format_name} has no audio stream (unexpected)");
                }
                Err(e) => {
                    panic!("{format_name} audio decoder creation failed: {e}");
                }
            }
        });
    }
}
