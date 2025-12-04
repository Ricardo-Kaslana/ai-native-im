# Requirements Document

## Introduction

本文档定义了 AI 原生即时通讯应用的媒体处理系统需求。该系统负责处理用户上传的图片和视频，包括格式转换、压缩优化、安全存储、快速分发和缩略图生成。系统必须在保证媒体质量的同时，优化存储成本和传输效率，符合应用的高性能要求。

## Glossary

- **Media Processing System**: 媒体处理系统，负责媒体文件的上传、处理、存储和分发的核心服务
- **Media Upload**: 媒体上传，用户将图片或视频文件上传到服务器的过程
- **Image Compression**: 图片压缩，将图片转换为更小文件大小的过程
- **Video Compression**: 视频压缩，使用编码器减小视频文件大小的过程
- **Thumbnail**: 缩略图，媒体文件的小尺寸预览图
- **WebP**: 现代图片格式，提供更好的压缩率
- **H.264/H.265**: 视频编码标准，用于高效视频压缩
- **CRF**: Constant Rate Factor，视频质量控制参数
- **Object Storage**: 对象存储，用于存储媒体文件的云存储服务（如 AWS S3、MinIO、腾讯云 COS、阿里云 OSS、CloudFlare R2）
- **Storage Provider**: 存储提供商，实现对象存储接口的具体云服务商
- **MinIO**: 开源的 S3 兼容对象存储服务，可自托管
- **Tencent COS**: 腾讯云对象存储服务
- **Aliyun OSS**: 阿里云对象存储服务
- **CloudFlare R2**: CloudFlare 的 S3 兼容对象存储服务
- **CDN**: Content Delivery Network，内容分发网络，加速媒体文件访问
- **Presigned URL**: 预签名 URL，临时授权的文件访问链接
- **Storage Adapter**: 存储适配器，统一不同存储提供商的接口实现
- **Media Metadata**: 媒体元数据，包括文件大小、尺寸、时长等信息
- **Upload Token**: 上传令牌，授权用户上传媒体的临时凭证
- **File Hash**: 文件哈希，用于去重和完整性验证的文件指纹
- **MIME Type**: 媒体类型标识符，如 image/jpeg, video/mp4
- **Aspect Ratio**: 宽高比，媒体文件的宽度与高度比例
- **Bitrate**: 比特率，视频或音频的数据传输速率

## Requirements

### Requirement 1

**User Story:** 作为用户，我希望能够上传图片到聊天中，以便分享视觉内容。

#### Acceptance Criteria

1. WHEN a user requests to upload an image THEN the Media Processing System SHALL generate a presigned upload URL valid for 15 minutes
2. WHEN generating upload URL THEN the Media Processing System SHALL validate the file size does not exceed 10 MB
3. WHEN an image is uploaded THEN the Media Processing System SHALL validate the MIME type is image/jpeg, image/png, or image/gif
4. WHEN image upload completes THEN the Media Processing System SHALL return a media ID and public access URL
5. WHEN upload fails THEN the Media Processing System SHALL return a descriptive error message

### Requirement 2

**User Story:** 作为用户，我希望上传的图片能够自动压缩，以便节省流量和存储空间。

#### Acceptance Criteria

1. WHEN an image is uploaded THEN the Media Processing System SHALL convert the image to WebP format
2. WHEN converting to WebP THEN the Media Processing System SHALL maintain image quality at 85% compression level
3. WHEN the original image exceeds 1920 pixels on the longest edge THEN the Media Processing System SHALL resize it to 1920 pixels while maintaining aspect ratio
4. WHEN image processing completes THEN the Media Processing System SHALL store both the original and processed versions
5. WHEN serving images THEN the Media Processing System SHALL serve the WebP version by default and fall back to original if client doesn't support WebP

### Requirement 3

**User Story:** 作为用户，我希望能够快速预览图片，以便在加载完整图片前看到内容。

#### Acceptance Criteria

1. WHEN an image is processed THEN the Media Processing System SHALL generate a thumbnail with maximum dimension of 200 pixels
2. WHEN generating thumbnails THEN the Media Processing System SHALL maintain the original aspect ratio
3. WHEN a thumbnail is requested THEN the Media Processing System SHALL serve it with aggressive caching headers (7 days)
4. WHEN thumbnail generation fails THEN the Media Processing System SHALL fall back to serving the full image
5. WHEN serving thumbnails THEN the Media Processing System SHALL use WebP format for supported clients

### Requirement 4

**User Story:** 作为用户，我希望能够上传视频到聊天中，以便分享动态内容。

#### Acceptance Criteria

1. WHEN a user requests to upload a video THEN the Media Processing System SHALL generate a presigned upload URL valid for 30 minutes
2. WHEN generating upload URL THEN the Media Processing System SHALL validate the file size does not exceed 100 MB
3. WHEN a video is uploaded THEN the Media Processing System SHALL validate the MIME type is video/mp4, video/quicktime, or video/webm
4. WHEN video upload completes THEN the Media Processing System SHALL return a media ID and processing status
5. WHEN upload fails THEN the Media Processing System SHALL return a descriptive error message

### Requirement 5

**User Story:** 作为用户，我希望上传的视频能够自动压缩，以便减少存储和传输成本。

#### Acceptance Criteria

1. WHEN a video is uploaded THEN the Media Processing System SHALL compress it using H.264 or H.265 codec
2. WHEN compressing video THEN the Media Processing System SHALL use CRF value between 23 and 28 for quality control
3. WHEN the video resolution exceeds 1080p THEN the Media Processing System SHALL downscale it to 1080p while maintaining aspect ratio
4. WHEN video processing completes THEN the Media Processing System SHALL update the processing status to completed
5. WHEN video processing fails THEN the Media Processing System SHALL update the status to failed and log the error

### Requirement 6

**User Story:** 作为用户，我希望能够看到视频的缩略图，以便在播放前预览内容。

#### Acceptance Criteria

1. WHEN a video is processed THEN the Media Processing System SHALL extract a thumbnail from the first frame
2. WHEN extracting video thumbnail THEN the Media Processing System SHALL generate an image with maximum dimension of 400 pixels
3. WHEN video thumbnail is generated THEN the Media Processing System SHALL store it in WebP format
4. WHEN a video thumbnail is requested THEN the Media Processing System SHALL serve it with caching headers
5. WHEN thumbnail extraction fails THEN the Media Processing System SHALL use a default video placeholder image

### Requirement 7

**User Story:** 作为开发者，我希望系统能够防止重复上传相同文件，以便节省存储空间。

#### Acceptance Criteria

1. WHEN a file is uploaded THEN the Media Processing System SHALL calculate the SHA-256 hash of the file content
2. WHEN the file hash matches an existing file THEN the Media Processing System SHALL return the existing media ID without reprocessing
3. WHEN deduplication occurs THEN the Media Processing System SHALL increment a reference counter for the existing file
4. WHEN a deduplicated file is deleted THEN the Media Processing System SHALL decrement the reference counter
5. WHEN the reference counter reaches zero THEN the Media Processing System SHALL delete the file from storage

### Requirement 8

**User Story:** 作为用户，我希望能够快速访问媒体文件，以便获得流畅的聊天体验。

#### Acceptance Criteria

1. WHEN a media file is stored THEN the Media Processing System SHALL upload it to object storage with public read access
2. WHEN serving media files THEN the Media Processing System SHALL use CDN for content delivery
3. WHEN generating media URLs THEN the Media Processing System SHALL return CDN URLs for optimal performance
4. WHEN a media file is accessed THEN the Media Processing System SHALL set appropriate cache headers (30 days for immutable content)
5. WHEN CDN is unavailable THEN the Media Processing System SHALL fall back to direct object storage URLs

### Requirement 9

**User Story:** 作为系统管理员，我希望能够管理媒体文件的生命周期，以便控制存储成本。

#### Acceptance Criteria

1. WHEN a media file is uploaded THEN the Media Processing System SHALL record the upload timestamp and uploader ID
2. WHEN a media file is not referenced by any message for 90 days THEN the Media Processing System SHALL mark it for deletion
3. WHEN a file is marked for deletion THEN the Media Processing System SHALL move it to a cold storage tier
4. WHEN a file has been in cold storage for 30 days THEN the Media Processing System SHALL permanently delete it
5. WHEN lifecycle policies are applied THEN the Media Processing System SHALL log all deletion actions for audit

### Requirement 10

**User Story:** 作为用户，我希望系统能够验证上传的文件安全性，以便防止恶意文件传播。

#### Acceptance Criteria

1. WHEN a file is uploaded THEN the Media Processing System SHALL scan the file for malware using antivirus service
2. WHEN malware is detected THEN the Media Processing System SHALL reject the upload and notify the user
3. WHEN scanning a file THEN the Media Processing System SHALL complete the scan within 10 seconds for files under 10 MB
4. WHEN the antivirus service is unavailable THEN the Media Processing System SHALL allow upload but flag for later scanning
5. WHEN a flagged file is later found to be malicious THEN the Media Processing System SHALL delete it and notify affected users

### Requirement 11

**User Story:** 作为用户，我希望能够查看媒体文件的元数据，以便了解文件信息。

#### Acceptance Criteria

1. WHEN a media file is processed THEN the Media Processing System SHALL extract and store metadata including file size, dimensions, and duration
2. WHEN retrieving media information THEN the Media Processing System SHALL return metadata along with the media URL
3. WHEN metadata extraction fails THEN the Media Processing System SHALL store basic information (file size, MIME type) and continue processing
4. WHEN serving media metadata THEN the Media Processing System SHALL cache it in Redis with 1-hour TTL
5. WHEN metadata is updated THEN the Media Processing System SHALL invalidate the cache

### Requirement 12

**User Story:** 作为开发者，我希望系统能够处理并发上传，以便支持多用户同时上传。

#### Acceptance Criteria

1. WHEN multiple users upload simultaneously THEN the Media Processing System SHALL process each upload in a separate async task
2. WHEN processing concurrent uploads THEN the Media Processing System SHALL limit concurrent processing to 10 tasks per server instance
3. WHEN the processing queue is full THEN the Media Processing System SHALL queue additional uploads and process them when capacity is available
4. WHEN an upload task fails THEN the Media Processing System SHALL retry up to 3 times with exponential backoff
5. WHEN all retries fail THEN the Media Processing System SHALL mark the upload as failed and notify the user

### Requirement 13

**User Story:** 作为用户，我希望能够删除我上传的媒体文件，以便管理我的内容。

#### Acceptance Criteria

1. WHEN a user requests to delete a media file THEN the Media Processing System SHALL verify the user is the original uploader
2. WHEN deleting a media file THEN the Media Processing System SHALL check if the file is referenced by any messages
3. WHEN a file is still referenced THEN the Media Processing System SHALL prevent deletion and return an error
4. WHEN a file is not referenced THEN the Media Processing System SHALL delete it from storage and database
5. WHEN deletion completes THEN the Media Processing System SHALL return a success response

### Requirement 14

**User Story:** 作为开发者，我希望系统能够记录媒体处理操作，以便监控和调试。

#### Acceptance Criteria

1. WHEN a media file is uploaded THEN the Media Processing System SHALL log the upload event with user ID, file size, and timestamp
2. WHEN media processing starts THEN the Media Processing System SHALL log the processing start time and file ID
3. WHEN media processing completes THEN the Media Processing System SHALL log the completion time and processing duration
4. WHEN processing fails THEN the Media Processing System SHALL log the error with full context and stack trace
5. WHEN serving media files THEN the Media Processing System SHALL track access metrics including request count and bandwidth usage

### Requirement 15

**User Story:** 作为系统架构师，我希望媒体处理系统的各个组件职责清晰分离，以便系统易于维护和扩展。

#### Acceptance Criteria

1. WHEN the system is initialized THEN the Media Processing System SHALL separate concerns into distinct modules: upload handler, image processor, video processor, and storage manager
2. WHEN a module interface is defined THEN the Media Processing System SHALL use Rust traits to enforce contract boundaries
3. WHEN the upload handler receives a file THEN the Media Processing System SHALL delegate processing to the appropriate processor module
4. WHEN the processor completes THEN the Media Processing System SHALL delegate storage to the storage manager module
5. WHEN any module is replaced or updated THEN the Media Processing System SHALL continue functioning without modifications to other modules

### Requirement 16

**User Story:** 作为系统管理员，我希望能够配置不同的云存储提供商，以便根据成本、性能和地域需求选择最合适的存储方案。

#### Acceptance Criteria

1. WHEN the system starts THEN the Media Processing System SHALL read storage provider configuration from environment variables
2. WHEN storage provider is set to AWS S3 THEN the Media Processing System SHALL use AWS S3 SDK for storage operations
3. WHEN storage provider is set to MinIO THEN the Media Processing System SHALL use S3-compatible API with custom endpoint
4. WHEN storage provider is set to Tencent COS THEN the Media Processing System SHALL use Tencent Cloud SDK for storage operations
5. WHEN storage provider is set to Aliyun OSS THEN the Media Processing System SHALL use Aliyun SDK for storage operations
6. WHEN storage provider is set to CloudFlare R2 THEN the Media Processing System SHALL use S3-compatible API with R2 endpoint
7. WHEN an unsupported provider is configured THEN the Media Processing System SHALL fail to start and log a clear error message

### Requirement 17

**User Story:** 作为开发者，我希望存储提供商的切换不影响业务逻辑，以便轻松迁移或使用多个存储服务。

#### Acceptance Criteria

1. WHEN implementing storage operations THEN the Media Processing System SHALL define a unified StorageProvider trait
2. WHEN any storage provider is used THEN the Media Processing System SHALL support upload, download, delete, and presigned URL operations
3. WHEN switching storage providers THEN the Media Processing System SHALL maintain the same API interface for all operations
4. WHEN a storage operation is called THEN the Media Processing System SHALL delegate to the configured provider implementation
5. WHEN adding a new storage provider THEN the Media Processing System SHALL only require implementing the StorageProvider trait

### Requirement 18

**User Story:** 作为系统管理员，我希望能够为不同的媒体类型配置不同的存储提供商，以便优化成本和性能。

#### Acceptance Criteria

1. WHEN configuring storage THEN the Media Processing System SHALL support separate provider configuration for images and videos
2. WHEN uploading an image THEN the Media Processing System SHALL use the configured image storage provider
3. WHEN uploading a video THEN the Media Processing System SHALL use the configured video storage provider
4. WHEN both media types use the same provider THEN the Media Processing System SHALL reuse the same provider instance
5. WHEN retrieving media URLs THEN the Media Processing System SHALL generate URLs based on the provider used for that file

### Requirement 19

**User Story:** 作为开发者，我希望系统能够处理不同存储提供商的特定配置需求，以便正确连接和使用各个服务。

#### Acceptance Criteria

1. WHEN configuring AWS S3 THEN the Media Processing System SHALL require region, bucket name, access key, and secret key
2. WHEN configuring MinIO THEN the Media Processing System SHALL require endpoint URL, bucket name, access key, and secret key
3. WHEN configuring Tencent COS THEN the Media Processing System SHALL require region, bucket name, secret ID, and secret key
4. WHEN configuring Aliyun OSS THEN the Media Processing System SHALL require region, bucket name, access key ID, and access key secret
5. WHEN configuring CloudFlare R2 THEN the Media Processing System SHALL require account ID, bucket name, access key ID, and secret access key
6. WHEN any required configuration is missing THEN the Media Processing System SHALL fail to initialize and log the missing parameter

### Requirement 20

**User Story:** 作为用户，我希望无论使用哪个存储提供商，媒体文件的访问速度都能保持快速，以便获得一致的用户体验。

#### Acceptance Criteria

1. WHEN a storage provider supports CDN THEN the Media Processing System SHALL configure CDN for that provider
2. WHEN generating media URLs THEN the Media Processing System SHALL return CDN URLs if available for the provider
3. WHEN CDN is not configured THEN the Media Processing System SHALL return direct storage URLs
4. WHEN serving media files THEN the Media Processing System SHALL set appropriate cache headers regardless of provider
5. WHEN a provider's CDN is unavailable THEN the Media Processing System SHALL fall back to direct storage URLs
