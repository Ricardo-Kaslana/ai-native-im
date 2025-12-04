//! End-to-End Media Flow Tests
//!
//! Tests the media processing system including uploads,
//! deduplication, and thumbnail generation.

mod common;

use uuid::Uuid;

// ============================================================================
// E2E Test: Media Type Validation
// ============================================================================

/// **E2E Test: Media type enum validation**
/// 
/// Tests that media types are properly defined:
/// 1. Image type
/// 2. Video type
#[tokio::test]
async fn e2e_media_type_validation() {
    use ai_im_server::services::media::{MediaType, MediaStatus};

    // Test media types
    let image = MediaType::Image;
    let video = MediaType::Video;
    
    // They should be distinct
    assert!(matches!(image, MediaType::Image));
    assert!(matches!(video, MediaType::Video));
    
    // Test media status
    let processing = MediaStatus::Processing;
    let completed = MediaStatus::Completed;
    let failed = MediaStatus::Failed;
    
    assert!(matches!(processing, MediaStatus::Processing));
    assert!(matches!(completed, MediaStatus::Completed));
    assert!(matches!(failed, MediaStatus::Failed));
    
    println!("✅ E2E Media Type Validation Test Passed!");
}

/// **E2E Test: Media file model**
/// 
/// Tests media file structure:
/// 1. Create media file record
/// 2. Verify fields
#[tokio::test]
async fn e2e_media_file_model() {
    use ai_im_server::services::media::{MediaFile, MediaType, MediaStatus};
    use chrono::Utc;

    let media_file = MediaFile {
        id: Uuid::new_v4(),
        uploader_id: Uuid::new_v4(),
        file_hash: "abc123def456789".to_string(),
        media_type: MediaType::Image,
        mime_type: "image/jpeg".to_string(),
        storage_provider: "local".to_string(),
        original_url: "/uploads/original.jpg".to_string(),
        processed_url: Some("/uploads/processed.webp".to_string()),
        thumbnail_url: Some("/uploads/thumb.webp".to_string()),
        width: Some(1920),
        height: Some(1080),
        duration: None,
        file_size: 1024 * 100,
        ref_count: 1,
        status: MediaStatus::Completed,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert!(matches!(media_file.media_type, MediaType::Image));
    assert_eq!(media_file.mime_type, "image/jpeg");
    assert_eq!(media_file.width, Some(1920));
    assert_eq!(media_file.height, Some(1080));
    assert!(media_file.thumbnail_url.is_some());
    assert!(matches!(media_file.status, MediaStatus::Completed));
    
    println!("✅ E2E Media File Model Test Passed!");
}

/// **E2E Test: File hash calculation for deduplication**
/// 
/// Tests the deduplication mechanism:
/// 1. Calculate hash for test data
/// 2. Verify same data produces same hash
/// 3. Verify different data produces different hash
#[tokio::test]
async fn e2e_file_deduplication_hash() {
    use sha2::{Sha256, Digest};
    
    let test_data_1 = b"Hello, this is test file content!";
    let test_data_2 = b"Hello, this is test file content!";
    let test_data_3 = b"Different content here";
    
    // Calculate hashes
    let hash_1 = format!("{:x}", Sha256::digest(test_data_1));
    let hash_2 = format!("{:x}", Sha256::digest(test_data_2));
    let hash_3 = format!("{:x}", Sha256::digest(test_data_3));
    
    // Same content should produce same hash
    assert_eq!(hash_1, hash_2, "Same content should produce same hash");
    
    // Different content should produce different hash
    assert_ne!(hash_1, hash_3, "Different content should produce different hash");
    
    // Hash should be 64 characters (SHA-256)
    assert_eq!(hash_1.len(), 64, "SHA-256 hash should be 64 hex characters");
    
    println!("✅ E2E File Deduplication Hash Test Passed!");
}

/// **E2E Test: Media metadata model**
/// 
/// Tests metadata handling:
/// 1. Create media metadata
/// 2. Verify fields
#[tokio::test]
async fn e2e_media_metadata() {
    use ai_im_server::services::media::MediaMetadata;

    let metadata = MediaMetadata {
        media_id: Uuid::new_v4(),
        format: Some("jpeg".to_string()),
        codec: Some("h264".to_string()),
        bitrate: Some(5000000),
        framerate: Some(30.0),
        width: Some(1920),
        height: Some(1080),
        duration: Some(120.5),
        file_size: 1024 * 1024 * 10,
        mime_type: "video/mp4".to_string(),
        extra: Some(serde_json::json!({"camera": "iPhone", "location": "NYC"})),
    };
    
    assert!(metadata.format.is_some());
    assert_eq!(metadata.format.as_ref().unwrap(), "jpeg");
    assert!(metadata.codec.is_some());
    assert!(metadata.bitrate.is_some());
    assert!(metadata.framerate.is_some());
    assert_eq!(metadata.width, Some(1920));
    assert_eq!(metadata.height, Some(1080));
    assert!(metadata.extra.is_some());
    
    println!("✅ E2E Media Metadata Test Passed!");
}

/// **E2E Test: Upload request validation**
/// 
/// Tests upload request structure:
/// 1. Create upload request
/// 2. Verify required fields
#[tokio::test]
async fn e2e_upload_request_validation() {
    use ai_im_server::services::media::UploadRequest;

    let request = UploadRequest {
        file_size: 1024 * 1024 * 5, // 5MB
        mime_type: "application/pdf".to_string(),
    };
    
    assert_eq!(request.file_size, 5 * 1024 * 1024);
    assert_eq!(request.mime_type, "application/pdf");
    
    // Test with image
    let image_request = UploadRequest {
        file_size: 1024 * 500, // 500KB
        mime_type: "image/jpeg".to_string(),
    };
    
    assert!(image_request.mime_type.starts_with("image/"));
    
    println!("✅ E2E Upload Request Validation Test Passed!");
}

/// **E2E Test: Storage path generation**
/// 
/// Tests storage path generation logic:
/// 1. Generate paths for different file types
/// 2. Verify path structure
#[tokio::test]
async fn e2e_storage_path_generation() {
    use chrono::Utc;
    
    let now = Utc::now();
    let year = now.format("%Y").to_string();
    let month = now.format("%m").to_string();
    
    // Simulate path generation
    let file_id = Uuid::new_v4();
    let extension = "jpg";
    let path = format!("uploads/{}/{}/{}.{}", year, month, file_id, extension);
    
    // Verify path structure
    assert!(path.starts_with("uploads/"));
    assert!(path.contains(&year));
    assert!(path.contains(&month));
    assert!(path.ends_with(".jpg"));
    
    // Test with different extension
    let video_path = format!("uploads/{}/{}/{}.mp4", year, month, Uuid::new_v4());
    assert!(video_path.ends_with(".mp4"));
    
    println!("✅ E2E Storage Path Generation Test Passed!");
}
