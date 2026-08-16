use std::path::Path;

use base64::{Engine, engine::general_purpose::STANDARD};
use tracing::{debug, warn};

use crate::domain::ImageContent;

pub fn load_image_from_path(path: &str) -> Result<ImageContent, String> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(format!("Image file not found: {}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()));
    }

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let mime_type = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => return Err(format!("Unsupported image format: {}", extension)),
    };

    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read image file: {}", e))?;

    let base64_data = STANDARD.encode(&bytes);
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

    debug!(path = %path.display(), size = bytes.len(), "loaded image");

    Ok(ImageContent {
        url: data_url,
        detail: None,
    })
}

pub fn load_images_from_paths(paths: &[String]) -> Vec<ImageContent> {
    let mut images = Vec::new();

    for path in paths {
        match load_image_from_path(path) {
            Ok(image) => images.push(image),
            Err(e) => {
                warn!(path = %path, error = %e, "failed to load image");
            }
        }
    }

    images
}
