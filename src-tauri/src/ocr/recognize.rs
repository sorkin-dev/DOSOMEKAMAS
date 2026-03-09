use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rusty_tesseract::{Args, Image};
use std::io::Cursor;

use crate::errors::AppError;

/// Run Tesseract OCR on raw PNG image bytes and return the recognised text.
///
/// Tesseract is configured for the French language (`fra`) since Dofus uses
/// French item names and stat labels.
///
/// # Errors
/// Returns `AppError::Ocr` if:
/// - the image bytes cannot be decoded
/// - the Tesseract binary is not found on the system (`PATH`)
/// - Tesseract returns an error during recognition
pub fn recognize_text(image_data: &[u8]) -> Result<String, AppError> {
    // Decode the raw bytes into a DynamicImage using the `image` crate.
    let dynamic =
        image::load(Cursor::new(image_data), image::ImageFormat::Png)
            .or_else(|_| {
                // Fall back to format detection when the caller does not know the format.
                image::load_from_memory(image_data)
            })
            .map_err(|e| AppError::Ocr(format!("Failed to load image: {e}")))?;

    let img = Image::from_dynamic_image(&dynamic)
        .map_err(|e| AppError::Ocr(format!("Failed to prepare image for Tesseract: {e}")))?;

    let args = Args {
        lang: "fra".into(),
        ..Args::default()
    };

    let output = rusty_tesseract::image_to_string(&img, &args)
        .map_err(|e| AppError::Ocr(format!("Tesseract error: {e}")))?;

    Ok(output.trim().to_string())
}

/// Tauri command: accept a base64-encoded PNG image and return the OCR text.
#[tauri::command]
pub fn recognize_text_from_image(image_base64: String) -> Result<String, AppError> {
    let bytes = BASE64
        .decode(&image_base64)
        .map_err(|e| AppError::Ocr(format!("Invalid base64 image: {e}")))?;

    recognize_text(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recognize_invalid_bytes_returns_error() {
        // Garbage bytes should return an OCR error, not panic.
        let result = recognize_text(b"not a valid image");
        assert!(result.is_err());
    }

    #[test]
    fn test_recognize_invalid_base64_returns_error() {
        let result = recognize_text_from_image("!!!invalid_base64!!!".into());
        assert!(result.is_err());
    }
}
