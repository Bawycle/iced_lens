// SPDX-License-Identifier: MPL-2.0
use criterion::{criterion_group, criterion_main, Criterion};
use iced_lens::image_handler;
use std::hint::black_box; // Use std::hint::black_box
use std::path::PathBuf;

fn image_loading_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_loading");

    // Use a path to a dummy PNG image for benchmarking
    let current_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let image_path = current_dir.join("tests/data/sample.png");

    group.bench_function("load_sample_png", |b| {
        b.iter(|| {
            // Use black_box to prevent the compiler from optimizing away the call
            let _ = black_box(image_handler::load_image(&image_path).unwrap());
        });
    });

    group.finish();
}

criterion_group!(benches, image_loading_benchmark);
criterion_main!(benches);
