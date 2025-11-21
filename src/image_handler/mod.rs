use crate::error::{Error, Result};
use iced::widget::image;
use resvg::usvg;
use std::path::Path;
use tiny_skia;

#[derive(Debug, Clone)]
pub struct ImageData {
    pub handle: image::Handle,
}

pub fn load_image<P: AsRef<Path>>(path: P) -> Result<ImageData> {
    let path = path.as_ref();
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let handle = match extension.to_lowercase().as_str() {
        "svg" => {
            let svg_data = std::fs::read(path)?;
            let tree = usvg::Tree::from_data(&svg_data, &usvg::Options::default())
                .map_err(|e| Error::Svg(e.to_string()))?;

            let pixmap_size = tree.size().to_int_size();
            let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
            resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
            
            let png_data = pixmap.encode_png().map_err(|e| Error::Svg(e.to_string()))?;
            image::Handle::from_memory(png_data)
        }
        _ => {
            image::Handle::from_path(path)
        }
    };

    Ok(ImageData { handle })
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Svg(s)
    }
}


#[cfg(test)]
mod tests {
    // Tests will be updated later when we have sample data
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
