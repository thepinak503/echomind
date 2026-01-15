#[cfg(any(feature = "pdf", feature = "images"))]
mod multimodal_impl {
    use crate::error::{EchomindError, Result};
    use base64::{engine::general_purpose, Engine as _};
    use std::fs;
    use std::path::Path;
    use tokio::process::Command;

    #[cfg(feature = "images")]
    use image::GenericImageView;

    #[cfg(feature = "pdf")]
    use calamine::Reader;

    pub struct MultimodalManager;

    impl MultimodalManager {
        pub async fn capture_webcam_image() -> Result<String> {
            #[cfg(target_os = "windows")]
            let output_file = {
                let temp_dir = std::env::temp_dir();
                temp_dir
                    .join("echomind_webcam.jpg")
                    .to_string_lossy()
                    .to_string()
            };
            #[cfg(not(target_os = "windows"))]
            let output_file = "/tmp/echomind_webcam.jpg";

            if cfg!(target_os = "macos") {
                Command::new("imagesnap")
                    .arg(&output_file)
                    .output()
                    .await
                    .map_err(|e| {
                        EchomindError::Other(format!("Failed to capture webcam: {}", e))
                    })?;
            } else if cfg!(target_os = "linux") {
                Command::new("fswebcam")
                    .args(&["-r", "640x480", "--jpeg", "95", "-D", "1", &output_file])
                    .output()
                    .await
                    .map_err(|e| {
                        EchomindError::Other(format!("Failed to capture webcam: {}", e))
                    })?;
            } else if cfg!(target_os = "windows") {
                Command::new("powershell")
                    .args(&["-Command", &format!("Add-Type -AssemblyName System.Windows.Forms; Add-Type -AssemblyName System.Drawing; $camera = [System.Windows.Forms.Webcam]::new(); $camera.Capture('{}'); $camera.Dispose()", output_file)])
                    .output()
                    .await
                    .map_err(|e| EchomindError::Other(format!("Failed to capture webcam: {}", e)))?;
            }

            Ok(output_file.to_string())
        }

        pub async fn take_screenshot() -> Result<String> {
            #[cfg(target_os = "windows")]
            let output_file = {
                let temp_dir = std::env::temp_dir();
                temp_dir
                    .join("echomind_screenshot.png")
                    .to_string_lossy()
                    .to_string()
            };
            #[cfg(not(target_os = "windows"))]
            let output_file = "/tmp/echomind_screenshot.png";

            if cfg!(target_os = "macos") {
                Command::new("screencapture")
                    .arg(&output_file)
                    .output()
                    .await
                    .map_err(|e| {
                        EchomindError::Other(format!("Failed to take screenshot: {}", e))
                    })?;
            } else if cfg!(target_os = "linux") {
                Command::new("import")
                    .args(&["-window", "root", &output_file])
                    .output()
                    .await
                    .map_err(|e| {
                        EchomindError::Other(format!("Failed to take screenshot: {}", e))
                    })?;
            } else if cfg!(target_os = "windows") {
                Command::new("powershell")
                    .args(&["-Command", &format!("Add-Type -AssemblyName System.Windows.Forms; Add-Type -AssemblyName System.Drawing; $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds; $bmp = New-Object System.Drawing.Bitmap $bounds.width, $bounds.height; $graphics = [System.Drawing.Graphics]::FromImage($bmp); $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.size); $bmp.Save('{}', [System.Drawing.Imaging.ImageFormat]::Png); $graphics.Dispose(); $bmp.Dispose()", output_file)])
                    .output()
                    .await
                    .map_err(|e| EchomindError::Other(format!("Failed to take screenshot: {}", e)))?;
            }

            Ok(output_file.to_string())
        }

        #[cfg(feature = "pdf")]
        pub fn process_pdf(_file_path: &str) -> Result<String> {
            Err(EchomindError::Other(
                "PDF processing not yet fully implemented".to_string(),
            ))
        }

        #[cfg(not(feature = "pdf"))]
        pub fn process_pdf(_file_path: &str) -> Result<String> {
            Err(EchomindError::Other(
                "PDF support not enabled. Rebuild with 'cargo build --features pdf'".to_string(),
            ))
        }

        pub fn process_office_document(file_path: &str) -> Result<String> {
            let extension = Path::new(file_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();

            match extension.as_str() {
                "xlsx" | "xls" => {
                    #[cfg(feature = "images")]
                    {
                        Ok(format!(
                            "[Excel file: {} - content extraction available in future update]",
                            file_path
                        ))
                    }
                    #[cfg(not(feature = "images"))]
                    {
                        Err(EchomindError::Other(
                            "Image processing support not enabled. Rebuild with 'cargo build --features images'".to_string()
                        ))
                    }
                }
                "docx" => Ok("DOCX content extraction not yet implemented".to_string()),
                "pptx" => Ok("PPTX content extraction not yet implemented".to_string()),
                _ => Err(EchomindError::Other(format!(
                    "Unsupported office document format: {}",
                    extension
                ))),
            }
        }

        #[cfg(feature = "images")]
        pub fn process_image(file_path: &str) -> Result<String> {
            let img = image::open(file_path)
                .map_err(|e| EchomindError::FileError(format!("Failed to open image: {}", e)))?;

            let rgb_img = img.to_rgb8();
            let (width, height) = rgb_img.dimensions();
            let info = format!("Image: {}x{} pixels", width, height);

            Ok(info)
        }

        #[cfg(not(feature = "images"))]
        pub fn process_image(_file_path: &str) -> Result<String> {
            Err(EchomindError::Other(
                "Image processing support not enabled. Rebuild with 'cargo build --features images'".to_string()
            ))
        }

        pub fn process_batch_images(directory: &str) -> Result<Vec<String>> {
            let mut results = Vec::new();

            for entry in fs::read_dir(directory)
                .map_err(|e| EchomindError::FileError(format!("Failed to read directory: {}", e)))?
            {
                let entry = entry.map_err(|e| {
                    EchomindError::FileError(format!("Failed to read entry: {}", e))
                })?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if let Some(ext_str) = extension.to_str() {
                            if ["jpg", "jpeg", "png", "gif", "bmp", "webp"]
                                .contains(&ext_str.to_lowercase().as_str())
                            {
                                let path_str = path.to_string_lossy().to_string();
                                results.push(path_str);
                            }
                        }
                    }
                }
            }

            Ok(results)
        }

        pub fn image_to_base64(file_path: &str) -> Result<String> {
            let image_data = fs::read(file_path)
                .map_err(|e| EchomindError::FileError(format!("Failed to read image: {}", e)))?;

            Ok(general_purpose::STANDARD.encode(&image_data))
        }

        #[cfg(feature = "images")]
        pub fn resize_image(file_path: &str, max_width: u32, max_height: u32) -> Result<String> {
            let img = image::open(file_path)
                .map_err(|e| EchomindError::FileError(format!("Failed to open image: {}", e)))?;

            let (original_width, original_height) = img.dimensions();

            let (new_width, new_height) =
                if original_width > max_width || original_height > max_height {
                    let width_ratio = max_width as f32 / original_width as f32;
                    let height_ratio = max_height as f32 / original_height as f32;
                    let ratio = width_ratio.min(height_ratio);

                    (
                        (original_width as f32 * ratio).round() as u32,
                        (original_height as f32 * ratio).round() as u32,
                    )
                } else {
                    (original_width, original_height)
                };

            let resized_img =
                img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

            let output_path = format!(
                "{}_resized.{}",
                file_path.rsplit('.').next().unwrap_or(file_path),
                file_path.rsplit('.').nth(1).unwrap_or("jpg")
            );

            resized_img.save(&output_path).map_err(|e| {
                EchomindError::FileError(format!("Failed to save resized image: {}", e))
            })?;

            Ok(output_path)
        }

        #[cfg(not(feature = "images"))]
        pub fn resize_image(_file_path: &str, _max_width: u32, _max_height: u32) -> Result<String> {
            Err(EchomindError::Other(
                "Image processing support not enabled. Rebuild with 'cargo build --features images'".to_string()
            ))
        }

        pub fn extract_text_with_ocr(_file_path: &str) -> Result<String> {
            Ok("OCR text extraction not yet implemented".to_string())
        }
    }
}

#[cfg(not(any(feature = "pdf", feature = "images")))]
mod multimodal_stub {
    use crate::error::{EchomindError, Result};

    pub struct MultimodalManager;

    impl MultimodalManager {
        pub async fn capture_webcam_image() -> Result<String> {
            Err(EchomindError::Other(
                "Multimodal features not enabled".to_string(),
            ))
        }

        pub async fn take_screenshot() -> Result<String> {
            Err(EchomindError::Other(
                "Multimodal features not enabled".to_string(),
            ))
        }

        pub fn process_pdf(_file_path: &str) -> Result<String> {
            Err(EchomindError::Other(
                "PDF support not enabled. Rebuild with 'cargo build --features pdf'".to_string(),
            ))
        }

        pub fn process_office_document(_file_path: &str) -> Result<String> {
            Err(EchomindError::Other(
                "Multimodal features not enabled".to_string(),
            ))
        }

        pub fn process_image(_file_path: &str) -> Result<String> {
            Err(EchomindError::Other(
                "Image features not enabled. Rebuild with 'cargo build --features images'"
                    .to_string(),
            ))
        }

        pub fn process_batch_images(_directory: &str) -> Result<Vec<String>> {
            Err(EchomindError::Other(
                "Multimodal features not enabled".to_string(),
            ))
        }

        pub fn image_to_base64(_file_path: &str) -> Result<String> {
            Err(EchomindError::Other(
                "Multimodal features not enabled".to_string(),
            ))
        }

        pub fn resize_image(_file_path: &str, _max_width: u32, _max_height: u32) -> Result<String> {
            Err(EchomindError::Other(
                "Image features not enabled".to_string(),
            ))
        }

        pub fn extract_text_with_ocr(_file_path: &str) -> Result<String> {
            Err(EchomindError::Other(
                "Multimodal features not enabled".to_string(),
            ))
        }
    }
}

// Public API
#[cfg(any(feature = "pdf", feature = "images"))]
pub use multimodal_impl::MultimodalManager;

#[cfg(not(any(feature = "pdf", feature = "images")))]
pub use multimodal_stub::MultimodalManager;
