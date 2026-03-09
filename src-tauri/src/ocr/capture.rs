use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use image::ImageFormat;
use std::io::Cursor;
use xcap::Monitor;

use crate::errors::AppError;

/// Capture a specific region of the screen and return it as a base64-encoded PNG.
///
/// # Arguments
/// * `x` – left edge of the region (screen coordinates)
/// * `y` – top edge of the region (screen coordinates)
/// * `width` – width of the region in pixels
/// * `height` – height of the region in pixels
pub fn capture_region(x: i32, y: i32, width: u32, height: u32) -> Result<String, AppError> {
    // Grab the primary monitor (or the one that contains the requested region).
    let monitors = Monitor::all().map_err(|e| AppError::ScreenCapture(e.to_string()))?;

    // Pick the monitor that most closely contains the requested region.
    let monitor = monitors
        .into_iter()
        .find(|m| {
            let mx = m.x();
            let my = m.y();
            let mw = m.width() as i32;
            let mh = m.height() as i32;
            x >= mx && y >= my && (x + width as i32) <= (mx + mw) && (y + height as i32) <= (my + mh)
        })
        .ok_or_else(|| AppError::ScreenCapture("No monitor found for the given region.".into()))?;

    // Capture the entire monitor as an image.
    let full_image = monitor
        .capture_image()
        .map_err(|e| AppError::ScreenCapture(e.to_string()))?;

    // Translate screen coordinates to monitor-local coordinates.
    let monitor_x = monitor.x();
    let monitor_y = monitor.y();
    let local_x = (x - monitor_x).max(0) as u32;
    let local_y = (y - monitor_y).max(0) as u32;

    let img = image::DynamicImage::ImageRgba8(full_image);
    let cropped = img.crop_imm(local_x, local_y, width, height);

    // Encode to PNG and return as base64.
    let mut buf = Cursor::new(Vec::new());
    cropped
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| AppError::ScreenCapture(e.to_string()))?;

    Ok(BASE64.encode(buf.into_inner()))
}

/// Tauri command: capture a screen region and return the PNG as a base64 string.
#[tauri::command]
pub fn capture_screen_region(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<String, AppError> {
    capture_region(x, y, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_invalid_region_returns_error() {
        // Requesting a region outside any monitor should return an error rather than panic.
        let result = capture_region(99999, 99999, 9999, 9999);
        assert!(result.is_err());
    }
}
