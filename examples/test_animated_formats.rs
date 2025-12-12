// SPDX-License-Identifier: MPL-2.0
//! Test handling of animated formats (GIF, WebP)

use iced_lens::media::{detect_media_type, load_image, MediaType};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_animated_file.gif|webp>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    println!("🔍 Test du fichier: {}\n", file_path);

    // Test 1: Détection du type
    println!("1️⃣  Détection du type de média:");
    match detect_media_type(file_path) {
        Some(MediaType::Image) => println!("   ✅ Détecté comme Image (statique ou 1 frame)"),
        Some(MediaType::Video) => println!("   ✅ Détecté comme Vidéo (animé, multiple frames)"),
        None => println!("   ❌ Format non reconnu"),
    }

    // Test 2: Chargement de l'image
    println!("\n2️⃣  Chargement de l'image:");
    match load_image(file_path) {
        Ok(img_data) => {
            println!("   ✅ Chargement réussi");
            println!("   📐 Dimensions: {}x{}", img_data.width, img_data.height);
            println!("\n📝 Note:");
            println!("   - Fichiers statiques (1 frame): chargés normalement comme images");
            println!("   - Fichiers animés (>1 frame): détectés comme vidéos");
            println!("   - La lecture d'animation sera implémentée dans les phases suivantes");
        }
        Err(e) => {
            println!("   ❌ Erreur de chargement: {}", e);
        }
    }
}
