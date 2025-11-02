use crate::error::{EchomindError, Result};
use base64::{Engine as _, engine::general_purpose};
use image::{DynamicImage, ImageFormat};
use pdf::file::File as PdfFile;
use pdf::content::*;
use pdf::primitive::Primitive;
use pdf::reader::Reader;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use tokio::process::Command;

pub struct MultimodalManager;

impl MultimodalManager {
    pub async fn capture_webcam_image() -> Result<String> {
        let output_file = "/tmp/echomind_webcam.jpg";
        
        if cfg!(target_os = "macos") {
            Command::new("imagesnap")
                .arg(output_file)
                .output()
                .await
                .map_err(|e| EchomindError::Other(format!("Failed to capture webcam: {}", e)))?;
        } else if cfg!(target_os = "linux") {
            Command::new("fswebcam")
                .args(&["-r", "640x480", "--jpeg", "95", "-D", "1", output_file])
                .output()
                .await
                .map_err(|e| EchomindError::Other(format!("Failed to capture webcam: {}", e)))?;
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
        let output_file = "/tmp/echomind_screenshot.png";
        
        if cfg!(target_os = "macos") {
            Command::new("screencapture")
                .arg(output_file)
                .output()
                .await
                .map_err(|e| EchomindError::Other(format!("Failed to take screenshot: {}", e)))?;
        } else if cfg!(target_os = "linux") {
            Command::new("import")
                .args(&["-window", "root", output_file])
                .output()
                .await
                .map_err(|e| EchomindError::Other(format!("Failed to take screenshot: {}", e)))?;
        } else if cfg!(target_os = "windows") {
            Command::new("powershell")
                .args(&["-Command", &format!("Add-Type -AssemblyName System.Windows.Forms; Add-Type -AssemblyName System.Drawing; $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds; $bmp = New-Object System.Drawing.Bitmap $bounds.width, $bounds.height; $graphics = [System.Drawing.Graphics]::FromImage($bmp); $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.size); $bmp.Save('{}', [System.Drawing.Imaging.ImageFormat]::Png); $graphics.Dispose(); $bmp.Dispose()", output_file)])
                .output()
                .await
                .map_err(|e| EchomindError::Other(format!("Failed to take screenshot: {}", e)))?;
        }

        Ok(output_file.to_string())
    }

    pub fn process_pdf(file_path: &str) -> Result<String> {
        let file = fs::File::open(file_path)
            .map_err(|e| EchomindError::FileError(format!("Failed to open PDF: {}", e)))?;
        
        let mut reader = Reader::new(BufReader::new(file))
            .map_err(|e| EchomindError::Other(format!("Failed to read PDF: {}", e)))?;
        
        let mut text_content = String::new();
        
        for page in reader.pages() {
            let page = page.map_err(|e| EchomindError::Other(format!("Failed to read page: {}", e)))?;
            
            let content = page.contents()
                .map_err(|e| EchomindError::Other(format!("Failed to get page content: {}", e)))?;
            
            let operations = content.operations()
                .map_err(|e| EchomindError::Other(format!("Failed to parse operations: {}", e)))?;
            
            for op in operations {
                match op.operator {
                    Operator::Tj => {
                        if let Some(Primitive::String(text)) = op.operands.get(0) {
                            text_content.push_str(&text.to_string_lossy());
                            text_content.push(' ');
                        }
                    }
                    Operator::TJ => {
                        if let Some(Primitive::Array(arr)) = op.operands.get(0) {
                            for item in arr {
                                if let Primitive::String(text) = item {
                                    text_content.push_str(&text.to_string_lossy());
                                    text_content.push(' ');
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Ok(text_content)
    }

    pub fn process_office_document(file_path: &str) -> Result<String> {
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "xlsx" | "xls" => {
                let mut excel_data: Vec<Vec<String>> = Vec::new();
                let mut workbook = calamine::open_workbook(file_path)
                    .map_err(|e| EchomindError::Other(format!("Failed to open Excel file: {}", e)))?;
                
                if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
                    for row in range.rows() {
                        let row_data: Vec<String> = row.iter()
                            .map(|cell| cell.to_string())
                            .collect();
                        excel_data.push(row_data);
                    }
                }
                
                let mut text = String::new();
                for row in excel_data {
                    text.push_str(&row.join("\t"));
                    text.push('\n');
                }
                
                Ok(text)
            }
            "docx" => {
                // For DOCX, we'd need a library like docx-rs
                // For now, return placeholder
                Ok("DOCX content extraction not yet implemented".to_string())
            }
            "pptx" => {
                // For PPTX, we'd need a library like pptx-rs
                // For now, return placeholder
                Ok("PPTX content extraction not yet implemented".to_string())
            }
            _ => Err(EchomindError::Other(format!("Unsupported office document format: {}", extension))),
        }
    }

    pub fn process_image(file_path: &str) -> Result<String> {
        let img = image::open(file_path)
            .map_err(|e| EchomindError::FileError(format!("Failed to open image: {}", e)))?;
        
        // Convert to RGB if necessary
        let rgb_img = img.to_rgb8();
        
        // For now, return basic image info
        let (width, height) = rgb_img.dimensions();
        let info = format!("Image: {}x{} pixels", width, height);
        
        // In a real implementation, you might want to:
        // 1. Resize the image for API limits
        // 2. Convert to base64 for API transmission
        // 3. Extract text using OCR if needed
        
        Ok(info)
    }

    pub fn process_batch_images(directory: &str) -> Result<Vec<String>> {
        let mut results = Vec::new();
        
        for entry in fs::read_dir(directory)
            .map_err(|e| EchomindError::FileError(format!("Failed to read directory: {}", e)))? {
            let entry = entry.map_err(|e| EchomindError::FileError(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if let Some(ext_str) = extension.to_str() {
                        if ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext_str.to_lowercase().as_str()) {
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

    pub fn resize_image(file_path: &str, max_width: u32, max_height: u32) -> Result<String> {
        let img = image::open(file_path)
            .map_err(|e| EchomindError::FileError(format!("Failed to open image: {}", e)))?;
        
        let (original_width, original_height) = img.dimensions();
        
        // Calculate new dimensions maintaining aspect ratio
        let (new_width, new_height) = if original_width > max_width || original_height > max_height {
            let width_ratio = max_width as f32 / original_width as f32;
            let height_ratio = max_height as f32 / original_height as f32;
            let ratio = width_ratio.min(height_ratio);
            
            (
                (original_width as f32 * ratio).round() as u32,
                (original_height as f32 * ratio).round() as u32
            )
        } else {
            (original_width, original_height)
        };
        
        let resized_img = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
        
        let output_path = format!("{}_resized.{}", 
            file_path.rsplit('.').next().unwrap_or(file_path),
            file_path.rsplit('.').nth(1).unwrap_or("jpg")
        );
        
        resized_img.save(&output_path)
            .map_err(|e| EchomindError::FileError(format!("Failed to save resized image: {}", e)))?;
        
        Ok(output_path)
    }

    pub fn extract_text_with_ocr(file_path: &str) -> Result<String> {
        // This would integrate with an OCR library like tesseract
        // For now, return placeholder
        Ok("OCR text extraction not yet implemented".to_string())
    }
}