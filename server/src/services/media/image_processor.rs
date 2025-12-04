use super::error::{MediaError, Result};
use super::models::ProcessedImage;
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;
use std::path::Path;
use tokio::fs;
use tracing::{debug, info};

/// Memory limit for image processing (50MB)
const MAX_IMAGE_MEMORY: usize = 50 * 1024 * 1024;

/// Image processing configuration
#[derive(Debug, Clone)]
pub struct ImageProcessorConfig {
    /// WebP quality (0-100)
    pub webp_quality: u8,
    /// Maximum dimension for processed images
    pub max_dimension: u32,
    /// Thumbnail maximum dimension
    pub thumbnail_dimension: u32,
}

impl Default for ImageProcessorConfig {
    fn default() -> Self {
        Self {
            webp_quality: 85,
            max_dimension: 1920,
            thumbnail_dimension: 200,
        }
    }
}

/// Image Processor - handles image compression and optimization
pub struct ImageProcessor {
    config: ImageProcessorConfig,
}

impl ImageProcessor {
    pub fn new() -> Self {
        Self {
            config: ImageProcessorConfig::default(),
        }
    }

    pub fn with_config(config: ImageProcessorConfig) -> Self {
        Self { config }
    }

    /// Process image: convert to WebP, resize, generate thumbnail
    pub async fn process_image(&self, input_path: &str) -> Result<ProcessedImage> {
        let start_time = std::time::Instant::now();
        info!("Processing image: {}", input_path);

        // Read the image file
        let image_data = fs::read(input_path)
            .await
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to read image: {}", e)))?;

        // Check memory limit
        if image_data.len() > MAX_IMAGE_MEMORY {
            return Err(MediaError::ProcessingFailed(
                format!("Image too large for processing: {} bytes", image_data.len())
            ));
        }

        // Load image in blocking task to avoid blocking async runtime
        let img = tokio::task::spawn_blocking(move || {
            image::load_from_memory(&image_data)
        })
        .await
        .map_err(|e| MediaError::ProcessingFailed(format!("Task join error: {}", e)))?
        .map_err(|e| MediaError::ProcessingFailed(format!("Failed to decode image: {}", e)))?;

        let (original_width, original_height) = img.dimensions();
        debug!(
            "Original image dimensions: {}x{}",
            original_width, original_height
        );

        // Resize if necessary
        let processed_img = self.resize_if_needed(&img);
        let (processed_width, processed_height) = processed_img.dimensions();

        // Generate thumbnail
        let thumbnail_img = self.generate_thumbnail(&img);

        // Convert to WebP
        let webp_data = self.convert_to_webp(&processed_img)?;
        let thumbnail_webp_data = self.convert_to_webp(&thumbnail_img)?;

        // Generate output paths
        let input_path_obj = Path::new(input_path);
        let stem = input_path_obj
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");
        let parent = input_path_obj
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        let webp_path = format!("{}/{}_processed.webp", parent, stem);
        let thumbnail_path = format!("{}/{}_thumb.webp", parent, stem);

        // Save processed files
        fs::write(&webp_path, &webp_data)
            .await
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to save WebP: {}", e)))?;

        fs::write(&thumbnail_path, &thumbnail_webp_data)
            .await
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to save thumbnail: {}", e)))?;

        let processing_time = start_time.elapsed();
        info!(
            "Image processed successfully: {}x{} -> {}x{}, WebP size: {} bytes, time: {:?}",
            original_width,
            original_height,
            processed_width,
            processed_height,
            webp_data.len(),
            processing_time
        );

        Ok(ProcessedImage {
            original_path: input_path.to_string(),
            webp_path,
            thumbnail_path,
            width: processed_width,
            height: processed_height,
            file_size: webp_data.len() as u64,
        })
    }

    /// Process image with parallel thumbnail generation
    pub async fn process_image_parallel(&self, input_path: &str) -> Result<ProcessedImage> {
        let start_time = std::time::Instant::now();
        info!("Processing image (parallel): {}", input_path);

        // Read the image file
        let image_data = fs::read(input_path)
            .await
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to read image: {}", e)))?;

        if image_data.len() > MAX_IMAGE_MEMORY {
            return Err(MediaError::ProcessingFailed(
                format!("Image too large for processing: {} bytes", image_data.len())
            ));
        }

        // Clone data for parallel processing
        let image_data_clone = image_data.clone();
        let config = self.config.clone();
        let config_clone = self.config.clone();

        // Process main image and thumbnail in parallel
        let (main_result, thumb_result) = tokio::join!(
            tokio::task::spawn_blocking(move || {
                Self::process_main_image_sync(&image_data, &config)
            }),
            tokio::task::spawn_blocking(move || {
                Self::process_thumbnail_sync(&image_data_clone, &config_clone)
            })
        );

        let (processed_img, webp_data, processed_width, processed_height) = main_result
            .map_err(|e| MediaError::ProcessingFailed(format!("Task join error: {}", e)))?
            .map_err(|e| MediaError::ProcessingFailed(e))?;

        let thumbnail_webp_data = thumb_result
            .map_err(|e| MediaError::ProcessingFailed(format!("Task join error: {}", e)))?
            .map_err(|e| MediaError::ProcessingFailed(e))?;

        // Generate output paths
        let input_path_obj = Path::new(input_path);
        let stem = input_path_obj
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");
        let parent = input_path_obj
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        let webp_path = format!("{}/{}_processed.webp", parent, stem);
        let thumbnail_path = format!("{}/{}_thumb.webp", parent, stem);

        // Save files in parallel
        let (save_main, save_thumb) = tokio::join!(
            fs::write(&webp_path, &webp_data),
            fs::write(&thumbnail_path, &thumbnail_webp_data)
        );

        save_main.map_err(|e| MediaError::ProcessingFailed(format!("Failed to save WebP: {}", e)))?;
        save_thumb.map_err(|e| MediaError::ProcessingFailed(format!("Failed to save thumbnail: {}", e)))?;

        let processing_time = start_time.elapsed();
        info!(
            "Image processed (parallel): {}x{}, WebP size: {} bytes, time: {:?}",
            processed_width, processed_height, webp_data.len(), processing_time
        );

        Ok(ProcessedImage {
            original_path: input_path.to_string(),
            webp_path,
            thumbnail_path,
            width: processed_width,
            height: processed_height,
            file_size: webp_data.len() as u64,
        })
    }

    /// Synchronous main image processing for spawn_blocking
    fn process_main_image_sync(
        image_data: &[u8],
        config: &ImageProcessorConfig,
    ) -> std::result::Result<(DynamicImage, Vec<u8>, u32, u32), String> {
        let img = image::load_from_memory(image_data)
            .map_err(|e| format!("Failed to decode image: {}", e))?;

        let (width, height) = img.dimensions();
        let max_dim = config.max_dimension;

        // Resize if needed
        let processed_img = if width <= max_dim && height <= max_dim {
            img
        } else {
            let (new_width, new_height) = if width > height {
                let ratio = max_dim as f64 / width as f64;
                (max_dim, (height as f64 * ratio) as u32)
            } else {
                let ratio = max_dim as f64 / height as f64;
                ((width as f64 * ratio) as u32, max_dim)
            };
            img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        };

        let (processed_width, processed_height) = processed_img.dimensions();

        // Convert to WebP
        let mut buffer = Cursor::new(Vec::new());
        processed_img.write_to(&mut buffer, ImageFormat::WebP)
            .map_err(|e| format!("Failed to encode WebP: {}", e))?;

        Ok((processed_img, buffer.into_inner(), processed_width, processed_height))
    }

    /// Synchronous thumbnail processing for spawn_blocking
    fn process_thumbnail_sync(
        image_data: &[u8],
        config: &ImageProcessorConfig,
    ) -> std::result::Result<Vec<u8>, String> {
        let img = image::load_from_memory(image_data)
            .map_err(|e| format!("Failed to decode image: {}", e))?;

        let (width, height) = img.dimensions();
        let thumb_dim = config.thumbnail_dimension;

        let (thumb_width, thumb_height) = if width > height {
            let ratio = thumb_dim as f64 / width as f64;
            (thumb_dim, (height as f64 * ratio).max(1.0) as u32)
        } else {
            let ratio = thumb_dim as f64 / height as f64;
            ((width as f64 * ratio).max(1.0) as u32, thumb_dim)
        };

        let thumbnail_img = img.thumbnail(thumb_width, thumb_height);

        let mut buffer = Cursor::new(Vec::new());
        thumbnail_img.write_to(&mut buffer, ImageFormat::WebP)
            .map_err(|e| format!("Failed to encode thumbnail WebP: {}", e))?;

        Ok(buffer.into_inner())
    }

    /// Resize image if it exceeds max dimension while maintaining aspect ratio
    fn resize_if_needed(&self, img: &DynamicImage) -> DynamicImage {
        let (width, height) = img.dimensions();
        let max_dim = self.config.max_dimension;

        if width <= max_dim && height <= max_dim {
            return img.clone();
        }

        // Calculate new dimensions maintaining aspect ratio
        let (new_width, new_height) = if width > height {
            let ratio = max_dim as f64 / width as f64;
            (max_dim, (height as f64 * ratio) as u32)
        } else {
            let ratio = max_dim as f64 / height as f64;
            ((width as f64 * ratio) as u32, max_dim)
        };

        debug!("Resizing image from {}x{} to {}x{}", width, height, new_width, new_height);

        img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
    }

    /// Generate thumbnail with max dimension
    fn generate_thumbnail(&self, img: &DynamicImage) -> DynamicImage {
        let (width, height) = img.dimensions();
        let thumb_dim = self.config.thumbnail_dimension;

        // Calculate thumbnail dimensions maintaining aspect ratio
        let (thumb_width, thumb_height) = if width > height {
            let ratio = thumb_dim as f64 / width as f64;
            (thumb_dim, (height as f64 * ratio).max(1.0) as u32)
        } else {
            let ratio = thumb_dim as f64 / height as f64;
            ((width as f64 * ratio).max(1.0) as u32, thumb_dim)
        };

        debug!(
            "Generating thumbnail: {}x{} -> {}x{}",
            width, height, thumb_width, thumb_height
        );

        img.thumbnail(thumb_width, thumb_height)
    }

    /// Convert image to WebP format with configured quality
    fn convert_to_webp(&self, img: &DynamicImage) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());

        // Use PNG as intermediate format since image crate's WebP encoder
        // may not be available in all configurations
        // For production, consider using libwebp bindings
        img.write_to(&mut buffer, ImageFormat::WebP)
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to encode WebP: {}", e)))?;

        Ok(buffer.into_inner())
    }

    /// Extract metadata from image
    pub fn extract_metadata(&self, img: &DynamicImage) -> ImageMetadata {
        let (width, height) = img.dimensions();
        let color_type = match img {
            DynamicImage::ImageLuma8(_) => "Grayscale",
            DynamicImage::ImageLumaA8(_) => "GrayscaleAlpha",
            DynamicImage::ImageRgb8(_) => "RGB",
            DynamicImage::ImageRgba8(_) => "RGBA",
            DynamicImage::ImageLuma16(_) => "Grayscale16",
            DynamicImage::ImageLumaA16(_) => "GrayscaleAlpha16",
            DynamicImage::ImageRgb16(_) => "RGB16",
            DynamicImage::ImageRgba16(_) => "RGBA16",
            DynamicImage::ImageRgb32F(_) => "RGB32F",
            DynamicImage::ImageRgba32F(_) => "RGBA32F",
            _ => "Unknown",
        };

        ImageMetadata {
            width,
            height,
            color_type: color_type.to_string(),
            aspect_ratio: width as f64 / height as f64,
        }
    }

    /// Process image from bytes (useful for in-memory processing)
    pub async fn process_image_bytes(
        &self,
        image_data: &[u8],
        output_dir: &str,
        file_id: &str,
    ) -> Result<ProcessedImage> {
        // Load image from bytes
        let img = image::load_from_memory(image_data)
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to decode image: {}", e)))?;

        let (original_width, original_height) = img.dimensions();

        // Resize if necessary
        let processed_img = self.resize_if_needed(&img);
        let (processed_width, processed_height) = processed_img.dimensions();

        // Generate thumbnail
        let thumbnail_img = self.generate_thumbnail(&img);

        // Convert to WebP
        let webp_data = self.convert_to_webp(&processed_img)?;
        let thumbnail_webp_data = self.convert_to_webp(&thumbnail_img)?;

        // Generate output paths
        let webp_path = format!("{}/{}_processed.webp", output_dir, file_id);
        let thumbnail_path = format!("{}/{}_thumb.webp", output_dir, file_id);

        // Ensure output directory exists
        fs::create_dir_all(output_dir)
            .await
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to create output dir: {}", e)))?;

        // Save processed files
        fs::write(&webp_path, &webp_data)
            .await
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to save WebP: {}", e)))?;

        fs::write(&thumbnail_path, &thumbnail_webp_data)
            .await
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to save thumbnail: {}", e)))?;

        info!(
            "Image processed: {}x{} -> {}x{}, size: {} bytes",
            original_width, original_height, processed_width, processed_height, webp_data.len()
        );

        Ok(ProcessedImage {
            original_path: format!("{}/{}_original", output_dir, file_id),
            webp_path,
            thumbnail_path,
            width: processed_width,
            height: processed_height,
            file_size: webp_data.len() as u64,
        })
    }
}

/// Image metadata
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub color_type: String,
    pub aspect_ratio: f64,
}

impl Default for ImageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = ImageProcessorConfig::default();
        assert_eq!(config.webp_quality, 85);
        assert_eq!(config.max_dimension, 1920);
        assert_eq!(config.thumbnail_dimension, 200);
    }

    #[test]
    fn test_processor_creation() {
        let processor = ImageProcessor::new();
        assert_eq!(processor.config.webp_quality, 85);
    }
}
