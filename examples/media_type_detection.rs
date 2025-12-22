// SPDX-License-Identifier: MPL-2.0
//! Test media type detection with real file paths.

use iced_lens::media::{detect_media_type, MediaType};

fn main() {
    println!("🔍 Test de détection de type de média\n");

    let test_files = vec![
        // Images
        ("photo.jpg", Some(MediaType::Image)),
        ("image.PNG", Some(MediaType::Image)),
        ("graphic.svg", Some(MediaType::Image)),
        ("screenshot.webp", Some(MediaType::Image)),
        ("icon.ico", Some(MediaType::Image)),
        ("diagram.tiff", Some(MediaType::Image)),
        // Vidéos
        ("video.mp4", Some(MediaType::Video)),
        ("movie.AVI", Some(MediaType::Video)),
        ("clip.mkv", Some(MediaType::Video)),
        ("animation.webm", Some(MediaType::Video)),
        ("recording.MOV", Some(MediaType::Video)),
        ("stream.m4v", Some(MediaType::Video)),
        // Non supportés
        ("document.pdf", None),
        ("archive.zip", None),
        ("text.txt", None),
        // Chemins complets
        ("/home/user/videos/vacation.mp4", Some(MediaType::Video)),
        ("/home/user/photos/family.jpg", Some(MediaType::Image)),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (path, expected) in test_files {
        let detected = detect_media_type(path);
        let result = if detected == expected {
            passed += 1;
            "✅"
        } else {
            failed += 1;
            "❌"
        };

        let type_str = match detected {
            Some(MediaType::Image) => "Image",
            Some(MediaType::Video) => "Vidéo",
            None => "Non supporté",
        };

        println!("{result} {path} → {type_str}");
    }

    println!("\n📊 Résultats: {passed} passés, {failed} échoués");

    if failed > 0 {
        std::process::exit(1);
    }
}
