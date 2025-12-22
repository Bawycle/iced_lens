// SPDX-License-Identifier: MPL-2.0
//! Benchmarks for media navigation operations.
//!
//! Measures the performance of:
//! - Directory scanning (finding all media files)
//! - Navigation operations (next/previous)
//! - Full navigation workflow (navigate + load image)

use criterion::{criterion_group, criterion_main, Criterion};
use iced_lens::config::SortOrder;
use iced_lens::media::{self, navigator::MediaNavigator};
use std::hint::black_box;
use std::path::PathBuf;

/// Get the path to the test data directory.
fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

/// Benchmark directory scanning performance.
///
/// Measures how long it takes to scan a directory and build the media list.
fn bench_scan_directory(c: &mut Criterion) {
    let mut group = c.benchmark_group("media_navigation");

    let test_dir = test_data_dir();
    let sample_image = test_dir.join("sample.png");

    group.bench_function("scan_directory", |b| {
        b.iter(|| {
            let mut navigator = MediaNavigator::new();
            navigator
                .scan_directory(&sample_image, SortOrder::Alphabetical)
                .unwrap();
            black_box(&navigator);
        });
    });

    group.finish();
}

/// Benchmark navigation operations (next/previous).
///
/// Measures the pure navigation time without image loading.
fn bench_navigate(c: &mut Criterion) {
    let mut group = c.benchmark_group("media_navigation");

    let test_dir = test_data_dir();
    let sample_image = test_dir.join("sample.png");

    // Pre-scan directory for navigation benchmarks
    let mut navigator = MediaNavigator::new();
    navigator
        .scan_directory(&sample_image, SortOrder::Alphabetical)
        .unwrap();

    group.bench_function("peek_next", |b| {
        b.iter(|| {
            let nav = navigator.clone();
            black_box(nav.peek_next());
        });
    });

    group.bench_function("peek_previous", |b| {
        b.iter(|| {
            let nav = navigator.clone();
            black_box(nav.peek_previous());
        });
    });

    group.bench_function("peek_and_confirm_next", |b| {
        b.iter(|| {
            let mut nav = navigator.clone();
            if let Some(next) = nav.peek_next() {
                nav.confirm_navigation(&next);
            }
            black_box(&nav);
        });
    });

    group.finish();
}

/// Benchmark the full navigation workflow.
///
/// Measures the complete user experience: navigate to next/previous + load image.
/// Uses the animated GIF which is a valid, larger file for realistic benchmarking.
fn bench_navigate_and_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("media_navigation");

    let test_dir = test_data_dir();
    // Use test_animated.gif as it's a valid, larger test file
    let animated_gif = test_dir.join("test_animated.gif");

    // Pre-scan directory starting from animated gif
    let mut navigator = MediaNavigator::new();
    navigator
        .scan_directory(&animated_gif, SortOrder::Alphabetical)
        .unwrap();

    // Navigate to test_static.gif (next valid file)
    let static_gif = test_dir.join("test_static.gif");

    group.bench_function("navigate_and_load_gif", |b| {
        b.iter(|| {
            // Measure loading a known valid file
            black_box(media::load_media(&static_gif).unwrap());
        });
    });

    // Also benchmark loading the sample.png (smallest valid image)
    let sample_png = test_dir.join("sample.png");

    group.bench_function("navigate_and_load_png", |b| {
        b.iter(|| {
            black_box(media::load_media(&sample_png).unwrap());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_scan_directory,
    bench_navigate,
    bench_navigate_and_load
);
criterion_main!(benches);
