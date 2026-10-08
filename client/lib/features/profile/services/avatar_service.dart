import 'dart:convert';
import 'dart:io';
import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import 'package:image_picker/image_picker.dart';
import 'package:image/image.dart' as img;

import '../../../core/network/api_client.dart';

/// 头像上传响应
class AvatarUploadResponse {
  final String mediaId;
  final String fileUrl;

  AvatarUploadResponse({
    required this.mediaId,
    required this.fileUrl,
  });

  factory AvatarUploadResponse.fromJson(Map<String, dynamic> json) {
    return AvatarUploadResponse(
      mediaId: json['media_id'] as String? ?? json['id'] as String? ?? '',
      fileUrl: json['file_url'] as String? ?? json['url'] as String? ?? '',
    );
  }
}

/// 头像上传服务 - 使用服务器 Media API
class AvatarService {
  static final AvatarService instance = AvatarService._();
  AvatarService._();

  final _picker = ImagePicker();
  final _apiClient = ApiClient.instance;

  /// 从相册选择图片
  Future<File?> pickImageFromGallery() async {
    try {
      final XFile? image = await _picker.pickImage(
        source: ImageSource.gallery,
        maxWidth: 1024,
        maxHeight: 1024,
      );
      if (image != null) {
        return File(image.path);
      }
      return null;
    } catch (e) {
      debugPrint('AvatarService: Failed to pick image - $e');
      return null;
    }
  }

  /// 从相机拍照
  Future<File?> pickImageFromCamera() async {
    try {
      final XFile? image = await _picker.pickImage(
        source: ImageSource.camera,
        maxWidth: 1024,
        maxHeight: 1024,
      );
      if (image != null) {
        return File(image.path);
      }
      return null;
    } catch (e) {
      debugPrint('AvatarService: Failed to take photo - $e');
      return null;
    }
  }

  /// 将图片文件转为 base64（不含 data: 前缀）
  /// 使用 image 包重新编码以修复颜色空间问题
  Future<String> _imageToBase64(File imageFile, String mimeType) async {
    try {
      // 读取原始图像
      final originalBytes = await imageFile.readAsBytes();
      img.Image? decodedImage;

      // 根据 MIME 类型解码
      if (mimeType == 'image/png') {
        decodedImage = img.decodePng(originalBytes);
      } else {
        // JPEG 和其他格式
        decodedImage = img.decodeImage(originalBytes);
      }

      if (decodedImage == null) {
        // 如果解码失败，直接使用原始字节
        debugPrint('AvatarService: Failed to decode image, using raw bytes');
        return base64Encode(originalBytes);
      }

      // 调整图像大小（保持宽高比，最大 512x512）
      img.Image resized = img.copyResize(
        decodedImage,
        width: 512,
        height: 512,
        interpolation: img.Interpolation.linear,
      );

      // 编码为 JPEG（去除 alpha 通道，避免颜色问题）
      final encodedBytes = img.encodeJpg(resized, quality: 85);

      debugPrint('AvatarService: Image re-encoded successfully, size: ${encodedBytes.length}');
      return base64Encode(encodedBytes);
    } catch (e) {
      debugPrint('AvatarService: Error processing image: $e, using raw bytes');
      // 如果处理失败，使用原始字节
      final bytes = await imageFile.readAsBytes();
      return base64Encode(bytes);
    }
  }

  /// 获取 MIME 类型
  String _getMimeType(String filePath) {
    final ext = filePath.split('.').last.toLowerCase();
    switch (ext) {
      case 'jpg':
      case 'jpeg':
        return 'image/jpeg';
      case 'png':
        return 'image/png';
      case 'gif':
        return 'image/gif';
      case 'webp':
        return 'image/webp';
      default:
        return 'image/jpeg';
    }
  }

  /// 上传头像到服务器 Media API
  Future<String> uploadAvatar(
    File imageFile, {
    void Function(int sent, int total)? onProgress,
  }) async {
    try {
      // 获取 MIME 类型
      final originalMimeType = _getMimeType(imageFile.path);
      final fileName = imageFile.path.split('/').last;

      // 使用 image 包重新编码并转换为 base64（修复颜色问题）
      final base64Data = await _imageToBase64(imageFile, originalMimeType);

      debugPrint('AvatarService: Uploading avatar, size: ${base64Data.length}');

      // 调用服务器 Media API（使用 ApiClient 的 dio 实例以支持自动 token 刷新）
      // 注意：我们统一使用 image/jpeg，因为 _imageToBase64 会编码为 JPEG
      final response = await _apiClient.dio.post(
        '/media/upload/base64',
        data: {
          'file_name': fileName.replaceAll(RegExp(r'\.(png|jpg|jpeg|gif|webp)$'), '.jpg'),
          'mime_type': 'image/jpeg',
          'data': base64Data,
        },
        options: Options(
          headers: {
            'Content-Type': 'application/json',
          },
        ),
      );

      if (response.statusCode == 200 || response.statusCode == 201) {
        final result = AvatarUploadResponse.fromJson(response.data);
        debugPrint('AvatarService: Upload successful: ${result.fileUrl}');
        return result.fileUrl;
      } else {
        throw Exception('Upload failed with status ${response.statusCode}');
      }
    } catch (e) {
      debugPrint('AvatarService: Upload failed - $e');
      rethrow;
    }
  }

  /// 选择并上传头像（一步完成）
  Future<String?> pickAndUploadAvatar({
    ImageSource source = ImageSource.gallery,
    void Function(int sent, int total)? onProgress,
  }) async {
    try {
      // 选择图片（不压缩，避免颜色问题）
      final XFile? image = await _picker.pickImage(
        source: source,
        maxWidth: 1024,
        maxHeight: 1024,
      );

      if (image == null) {
        debugPrint('AvatarService: No image selected');
        return null;
      }

      // 上传（内部会用 image 包正确处理）
      return await uploadAvatar(
        File(image.path),
        onProgress: onProgress,
      );
    } catch (e) {
      debugPrint('AvatarService: Pick and upload failed - $e');
      return null;
    }
  }
}
