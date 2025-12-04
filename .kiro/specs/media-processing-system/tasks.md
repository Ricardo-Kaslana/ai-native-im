# Implementation Plan

- [x] 1. Set up project structure and dependencies
  - Add media processing modules to Rust project
  - Add dependencies: image, ffmpeg, aws-sdk-s3, sha2
  - Add multi-cloud storage dependencies (SDKs for Tencent COS, Aliyun OSS)
  - Configure storage provider settings
  - Set up processing queue (Redis or in-memory)
  - _Requirements: 1.1, 4.1, 8.1, 16.1_

- [x] 2. Implement database schema
  - Create media_files table with storage_provider column
  - Create media_metadata table
  - Add indexes for performance (including storage_provider index)
  - _Requirements: 7.1, 11.1, 16.2-16.6_

- [x] 3. Implement Upload Handler
  - Generate presigned URLs using configured storage provider
  - Validate file size and MIME type
  - Create upload tokens
  - Handle upload completion
  - Record storage provider used for each upload
  - _Requirements: 1.1, 1.2, 1.3, 4.1, 4.2, 4.3, 18.2, 18.3_

- [x] 4. Implement Image Processor
  - Convert images to WebP (85% quality)
  - Resize images to 1920px max
  - Generate 200px thumbnails
  - Extract metadata
  - _Requirements: 2.1, 2.2, 2.3, 3.1, 11.1_

- [x] 5. Implement Video Processor
  - Compress videos with H.264/H.265 (CRF 23-28)
  - Downscale to 1080p if needed
  - Extract first frame as thumbnail (400px)
  - Extract metadata (duration, resolution)
  - _Requirements: 5.1, 5.2, 5.3, 6.1, 6.2, 11.1_

- [x] 6. Implement multi-cloud storage architecture
- [x] 6.1 Define StorageProvider trait
  - Define unified trait interface for all storage providers
  - Include methods: upload_file, generate_presigned_url, get_public_url, get_cdn_url, delete_file
  - _Requirements: 17.1, 17.2_

- [x] 6.2 Implement AWS S3 Provider
  - Implement StorageProvider trait for AWS S3
  - Configure region, bucket, access keys
  - Support CloudFront CDN integration
  - _Requirements: 16.2, 19.1_

- [x] 6.3 Implement MinIO Provider


  - Implement StorageProvider trait for MinIO
  - Use S3-compatible API with custom endpoint
  - Support self-hosted deployment
  - _Requirements: 16.3, 19.2_



- [x] 6.4 Implement Tencent COS Provider
  - Implement StorageProvider trait for Tencent COS (placeholder)
  - Configure region, bucket, secret ID/key
  - Support Tencent CDN integration
  - _Requirements: 16.4, 19.3_



- [x] 6.5 Implement Aliyun OSS Provider
  - Implement StorageProvider trait for Aliyun OSS (placeholder)
  - Configure region, bucket, access keys
  - Support Aliyun CDN integration


  - _Requirements: 16.5, 19.4_

- [x] 6.6 Implement CloudFlare R2 Provider
  - Implement StorageProvider trait for CloudFlare R2


  - Use S3-compatible API with R2 endpoint
  - Support custom domain configuration
  - _Requirements: 16.6, 19.5_



- [x] 6.7 Implement Storage Manager with provider routing
  - Create StorageManager that manages multiple providers
  - Support separate providers for images and videos
  - Route uploads to appropriate provider based on media type
  - Generate URLs based on provider used
  - Set cache headers (30 days)
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 13.4, 18.1, 18.2, 18.3, 18.5_

- [x]* 6.8 Write property tests for storage providers
  - **Property 17: Storage provider configuration validation**
  - **Property 18: Storage provider interface consistency**
  - **Property 19: Provider-specific upload routing**
  - **Property 20: URL generation consistency**
  - **Property 21: CDN fallback behavior**


  - **Validates: Requirements 16.1, 17.2, 18.2, 18.3, 18.5, 19.6, 20.2, 20.5**

- [x] 7. Implement Deduplication Manager
  - Calculate SHA-256 file hashes
  - Check for duplicate files
  - Manage reference counts


  - Handle file deletion when ref count = 0
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [x] 8. Implement malware scanning integration
  - Integrate with antivirus service


  - Scan files on upload
  - Handle scan results
  - Flag files for later scanning if service unavailable
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [x] 9. Implement concurrent processing
  - Create async processing queue



  - Limit concurrent tasks to 10 per instance
  - Implement retry logic (3 attempts with backoff)
  - Handle processing failures
  - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5_

- [x] 10. Implement lifecycle management
  - Track file usage and last access time
  - Mark unused files for deletion (90 days)
  - Move to cold storage
  - Permanently delete after 30 days in cold storage
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [x] 11. Implement metadata management
  - Extract and store media metadata
  - Cache metadata in Redis (1-hour TTL)
  - Invalidate cache on updates
  - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

- [x] 12. Implement access control
  - Verify uploader ownership for deletion
  - Check message references before deletion
  - Generate presigned URLs for private access
  - _Requirements: 13.1, 13.2, 13.3_

- [x] 13. Implement logging and monitoring
  - Log upload events
  - Log processing start/completion
  - Log errors with context
  - Track access metrics and bandwidth
  - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5_

- [x] 14. Create API endpoints
  - POST /media/upload/image - Request image upload
  - POST /media/upload/video - Request video upload
  - POST /media/complete - Complete upload
  - GET /media/:id - Get media info
  - GET /media/:id/thumbnail - Get thumbnail
  - DELETE /media/:id - Delete media
  - _Requirements: 1.1, 4.1, 11.2, 13.1_

- [x]* 15. Write unit tests
  - Test file validation
  - Test image processing
  - Test video processing
  - Test deduplication
  - Test storage operations for each provider
  - Test provider configuration validation
  - _Requirements: All_

- [x]* 16. Write integration tests
  - Test full upload-process-store flow for each provider
  - Test CDN delivery for each provider
  - Test deduplication end-to-end
  - Test lifecycle management
  - Test provider switching
  - Test multi-provider concurrent operations
  - _Requirements: All_

- [x] 17. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 18. Performance optimization
  - Optimize image processing
  - Optimize video compression
  - Optimize S3 uploads
  - Optimize CDN caching
  - _Requirements: 8.3, 12.1_

- [x] 19. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
