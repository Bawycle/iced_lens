// SPDX-License-Identifier: MPL-2.0
//! Benchmark media type detection performance

use iced_lens::media::detect_media_type;
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];

    // Get file size
    let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

    println!("🎯 Benchmark de détection: {file_path}");
    println!(
        "📦 Taille du fichier: {:.2} KB ({} bytes)",
        file_size as f64 / 1024.0,
        file_size
    );
    println!();

    // Warmup
    let _ = detect_media_type(file_path);

    // Benchmark with multiple runs
    let iterations = 10;
    let mut times = Vec::new();

    for i in 0..iterations {
        let start = Instant::now();
        let _result = detect_media_type(file_path);
        let duration = start.elapsed();
        times.push(duration);

        if i == 0 {
            println!("⏱️  Mesure {} (première): {:?}", i + 1, duration);
        }
    }

    // Calculate statistics
    let total: std::time::Duration = times.iter().sum();
    let avg = total / iterations as u32;
    let min = times.iter().min().unwrap();
    let max = times.iter().max().unwrap();

    println!();
    println!("📊 Résultats ({iterations} itérations):");
    println!("   Moyenne: {avg:?}");
    println!("   Min:     {min:?}");
    println!("   Max:     {max:?}");
    println!();

    // Check target
    let target_ms = 50;
    let avg_ms = avg.as_millis();

    if avg_ms < target_ms {
        println!("✅ Performance EXCELLENTE (< {target_ms}ms)");
        println!(
            "   {}x plus rapide que la cible!",
            target_ms as f64 / avg_ms as f64
        );
    } else if avg_ms < target_ms * 2 {
        println!("✓  Performance ACCEPTABLE (< {}ms)", target_ms * 2);
    } else {
        println!("⚠️  Performance LENTE (> {}ms)", target_ms * 2);
    }
}
