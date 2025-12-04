use super::error::{MediaError, Result};
use super::models::ProcessedVideo;
use std::path::Path;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;
use tracing::{debug, error, info};

/// Video processing configuration
#[derive(Debug, Clone)]
pub struct VideoProcessorConfig {
    /// CRF value for quality control (lower = better quality, larger file)
    pub crf: u8,
    /// Maximum resolution (height)
    pub max_resolution: u32,
    /// Thumbnail maximum dimension
    pub thumbnail_dimension: u32,
    /// Video codec (h264 or h265)
    pub codec: VideoCodec,
    /// Audio codec
    pub audio_codec: String,
    /// Preset for encoding speed/quality tradeoff
    pub preset: String,
}

#[derive(Debug, Clone, Copy)]
pub enum VideoCodec {
    H264,
    H265,
}

impl VideoCodec {
    pub fn as_ffmpeg_codec(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "libx264",
            VideoCodec::H265 => "libx265",
        }
    }
}

impl Default for VideoProcessorConfig {
    fn default() -> Self {
        Self {
            crf: 23,
            max_resolution: 1080,
            thumbnail_dimension: 400,
            codec: VideoCodec::H264,
            audio_codec: "aac".to_string(),
            preset: "medium".to_string(),
        }
    }
}

/// Video Processor - handles video compression and optimization
pub struct VideoProcessor {
    config: VideoProcessorConfig,
}

impl VideoProcessor {
    pub fn new() -> Self {
        Self {
            config: VideoProcessorConfig::default(),
        }
    }

    pub fn with_config(config: VideoProcessorConfig) -> Self {
        Self { config }
    }

    /// Process video: compress, downscale, generate thumbnail
    pub async fn process_video(&self, input_path: &str) -> Result<ProcessedVideo> {
        info!("Processing video: {}", input_path);

        // Verify input file exists
        if !Path::new(input_path).exists() {
            return Err(MediaError::ProcessingFailed(format!(
                "Input file not found: {}",
                input_path
            )));
        }

        // Get video metadata first
        let metadata = self.get_video_metadata(input_path).await?;
        debug!("Video metadata: {:?}", metadata);

        // Generate output paths
        let input_path_obj = Path::new(input_path);
        let stem = input_path_obj
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("video");
        let parent = input_path_obj
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        let compressed_path = format!("{}/{}_compressed.mp4", parent, stem);
        let thumbnail_path = format!("{}/{}_thumb.webp", parent, stem);

        // Compress video
        self.compress_video(input_path, &compressed_path, &metadata)
            .await?;

        // Extract thumbnail from first frame
        self.extract_thumbnail(input_path, &thumbnail_path).await?;

        // Get compressed file size
        let file_size = fs::metadata(&compressed_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        info!(
            "Video processed: {}x{}, duration: {:.2}s, size: {} bytes",
            metadata.width, metadata.height, metadata.duration, file_size
        );

        Ok(ProcessedVideo {
            original_path: input_path.to_string(),
            compressed_path,
            thumbnail_path,
            width: metadata.width.min(self.config.max_resolution * 16 / 9), // Approximate max width
            height: metadata.height.min(self.config.max_resolution),
            duration: metadata.duration,
            file_size,
        })
    }

    /// Get video metadata using ffprobe
    async fn get_video_metadata(&self, input_path: &str) -> Result<VideoMetadata> {
        let output = Command::new("ffprobe")
            .args([
                "-v", "quiet",
                "-print_format", "json",
                "-show_format",
                "-show_streams",
                input_path,
            ])
            .output()
            .await
            .map_err(|e| {
                MediaError::ProcessingFailed(format!("Failed to run ffprobe: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MediaError::ProcessingFailed(format!(
                "ffprobe failed: {}",
                stderr
            )));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
            MediaError::ProcessingFailed(format!("Failed to parse ffprobe output: {}", e))
        })?;

        // Extract video stream info
        let streams = json["streams"].as_array().ok_or_else(|| {
            MediaError::ProcessingFailed("No streams found in video".to_string())
        })?;

        let video_stream = streams
            .iter()
            .find(|s| s["codec_type"].as_str() == Some("video"))
            .ok_or_else(|| {
                MediaError::ProcessingFailed("No video stream found".to_string())
            })?;

        let width = video_stream["width"].as_u64().unwrap_or(0) as u32;
        let height = video_stream["height"].as_u64().unwrap_or(0) as u32;

        // Get duration from format
        let duration = json["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        // Get bitrate
        let bitrate = json["format"]["bit_rate"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        // Get framerate
        let framerate_str = video_stream["r_frame_rate"]
            .as_str()
            .unwrap_or("0/1");
        let framerate = parse_framerate(framerate_str);

        Ok(VideoMetadata {
            width,
            height,
            duration,
            bitrate,
            framerate,
            codec: video_stream["codec_name"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
        })
    }

    /// Compress video using ffmpeg
    async fn compress_video(
        &self,
        input_path: &str,
        output_path: &str,
        metadata: &VideoMetadata,
    ) -> Result<()> {
        let mut args = vec![
            "-i".to_string(),
            input_path.to_string(),
            "-c:v".to_string(),
            self.config.codec.as_ffmpeg_codec().to_string(),
            "-crf".to_string(),
            self.config.crf.to_string(),
            "-preset".to_string(),
            self.config.preset.clone(),
            "-c:a".to_string(),
            self.config.audio_codec.clone(),
            "-movflags".to_string(),
            "+faststart".to_string(),
        ];

        // Add scale filter if video exceeds max resolution
        if metadata.height > self.config.max_resolution {
            let scale_filter = format!(
                "scale=-2:{}",
                self.config.max_resolution
            );
            args.extend(["-vf".to_string(), scale_filter]);
        }

        // Output file
        args.extend(["-y".to_string(), output_path.to_string()]);

        debug!("Running ffmpeg with args: {:?}", args);

        let output = Command::new("ffmpeg")
            .args(&args)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                MediaError::ProcessingFailed(format!("Failed to run ffmpeg: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("ffmpeg compression failed: {}", stderr);
            return Err(MediaError::ProcessingFailed(format!(
                "Video compression failed: {}",
                stderr
            )));
        }

        info!("Video compressed successfully: {}", output_path);
        Ok(())
    }

    /// Extract thumbnail from video (first frame)
    async fn extract_thumbnail(&self, input_path: &str, output_path: &str) -> Result<()> {
        let scale_filter = format!(
            "scale={}:-1",
            self.config.thumbnail_dimension
        );

        let output = Command::new("ffmpeg")
            .args([
                "-i", input_path,
                "-vf", &scale_filter,
                "-vframes", "1",
                "-f", "image2",
                "-c:v", "libwebp",
                "-y", output_path,
            ])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                MediaError::ProcessingFailed(format!("Failed to extract thumbnail: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Thumbnail extraction failed: {}", stderr);
            return Err(MediaError::ProcessingFailed(format!(
                "Thumbnail extraction failed: {}",
                stderr
            )));
        }

        info!("Thumbnail extracted: {}", output_path);
        Ok(())
    }

    /// Process video from bytes (saves to temp file first)
    pub async fn process_video_bytes(
        &self,
        video_data: &[u8],
        output_dir: &str,
        file_id: &str,
    ) -> Result<ProcessedVideo> {
        // Ensure output directory exists
        fs::create_dir_all(output_dir)
            .await
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to create output dir: {}", e)))?;

        // Save to temp file
        let temp_path = format!("{}/{}_temp.mp4", output_dir, file_id);
        fs::write(&temp_path, video_data)
            .await
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to save temp file: {}", e)))?;

        // Process the video
        let result = self.process_video(&temp_path).await;

        // Clean up temp file
        let _ = fs::remove_file(&temp_path).await;

        result
    }
}

/// Video metadata
#[derive(Debug, Clone)]
pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub bitrate: u64,
    pub framerate: f64,
    pub codec: String,
}

/// Parse framerate string (e.g., "30/1" or "29.97")
fn parse_framerate(s: &str) -> f64 {
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 2 {
            let num: f64 = parts[0].parse().unwrap_or(0.0);
            let den: f64 = parts[1].parse().unwrap_or(1.0);
            if den > 0.0 {
                return num / den;
            }
        }
    }
    s.parse().unwrap_or(0.0)
}

impl Default for VideoProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = VideoProcessorConfig::default();
        assert_eq!(config.crf, 23);
        assert_eq!(config.max_resolution, 1080);
        assert_eq!(config.thumbnail_dimension, 400);
    }

    #[test]
    fn test_parse_framerate() {
        assert!((parse_framerate("30/1") - 30.0).abs() < 0.01);
        assert!((parse_framerate("30000/1001") - 29.97).abs() < 0.01);
        assert!((parse_framerate("25") - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_codec_ffmpeg_name() {
        assert_eq!(VideoCodec::H264.as_ffmpeg_codec(), "libx264");
        assert_eq!(VideoCodec::H265.as_ffmpeg_codec(), "libx265");
    }
}
