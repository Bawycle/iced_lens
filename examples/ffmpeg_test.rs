// Test simple : Initialiser FFmpeg et afficher les codecs disponibles
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialiser FFmpeg
    ffmpeg_next::init()?;

    println!("✅ FFmpeg initialisé avec succès!");
    println!("Version FFmpeg: {}", ffmpeg_next::format::version());

    // Lister quelques décodeurs vidéo courants
    println!("\n📹 Décodeurs vidéo disponibles:");
    for codec_name in &["h264", "hevc", "vp9", "av1", "mpeg4"] {
        if let Some(codec) = ffmpeg_next::decoder::find_by_name(codec_name) {
            println!("  ✓ {}: {}", codec_name.to_uppercase(), codec.description());
        } else {
            println!("  ✗ {}: non disponible", codec_name.to_uppercase());
        }
    }

    // Lister quelques décodeurs audio
    println!("\n🔊 Décodeurs audio disponibles:");
    for codec_name in &["aac", "mp3", "opus", "vorbis"] {
        if let Some(codec) = ffmpeg_next::decoder::find_by_name(codec_name) {
            println!("  ✓ {}: {}", codec_name.to_uppercase(), codec.description());
        } else {
            println!("  ✗ {}: non disponible", codec_name.to_uppercase());
        }
    }

    Ok(())
}
