import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import '../../../core/constants/spacing.dart';

/// Media picker bottom sheet
class MediaPicker extends StatelessWidget {
  final Function(List<XFile>) onMediaSelected;
  final bool allowMultiple;
  final bool allowVideo;

  const MediaPicker({
    super.key,
    required this.onMediaSelected,
    this.allowMultiple = true,
    this.allowVideo = true,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(Spacing.md),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Handle bar
            Container(
              width: 40,
              height: 4,
              margin: const EdgeInsets.only(bottom: Spacing.md),
              decoration: BoxDecoration(
                color: Colors.grey[400],
                borderRadius: BorderRadius.circular(2),
              ),
            ),

            // Options
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                _buildOption(
                  context,
                  icon: Icons.camera_alt,
                  label: '拍照',
                  onTap: () => _pickFromCamera(context, isVideo: false),
                ),
                _buildOption(
                  context,
                  icon: Icons.photo_library,
                  label: '相册',
                  onTap: () => _pickFromGallery(context, isVideo: false),
                ),
                if (allowVideo) ...[
                  _buildOption(
                    context,
                    icon: Icons.videocam,
                    label: '录像',
                    onTap: () => _pickFromCamera(context, isVideo: true),
                  ),
                  _buildOption(
                    context,
                    icon: Icons.video_library,
                    label: '视频',
                    onTap: () => _pickFromGallery(context, isVideo: true),
                  ),
                ],
              ],
            ),

            const SizedBox(height: Spacing.md),

            // Cancel button
            SizedBox(
              width: double.infinity,
              child: TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('取消'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildOption(
    BuildContext context, {
    required IconData icon,
    required String label,
    required VoidCallback onTap,
  }) {
    final theme = Theme.of(context);

    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(Spacing.radiusMd),
      child: Padding(
        padding: const EdgeInsets.all(Spacing.md),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 56,
              height: 56,
              decoration: BoxDecoration(
                color: theme.colorScheme.primaryContainer,
                shape: BoxShape.circle,
              ),
              child: Icon(
                icon,
                color: theme.colorScheme.primary,
                size: 28,
              ),
            ),
            const SizedBox(height: Spacing.xs),
            Text(
              label,
              style: theme.textTheme.labelMedium,
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _pickFromCamera(BuildContext context, {required bool isVideo}) async {
    final picker = ImagePicker();
    final XFile? file;

    if (isVideo) {
      file = await picker.pickVideo(
        source: ImageSource.camera,
        maxDuration: const Duration(minutes: 5),
      );
    } else {
      file = await picker.pickImage(
        source: ImageSource.camera,
        imageQuality: 85,
      );
    }

    if (file != null && context.mounted) {
      Navigator.pop(context);
      onMediaSelected([file]);
    }
  }

  Future<void> _pickFromGallery(BuildContext context, {required bool isVideo}) async {
    final picker = ImagePicker();

    if (isVideo) {
      final file = await picker.pickVideo(source: ImageSource.gallery);
      if (file != null && context.mounted) {
        Navigator.pop(context);
        onMediaSelected([file]);
      }
    } else {
      if (allowMultiple) {
        final files = await picker.pickMultiImage(imageQuality: 85);
        if (files.isNotEmpty && context.mounted) {
          Navigator.pop(context);
          onMediaSelected(files);
        }
      } else {
        final file = await picker.pickImage(
          source: ImageSource.gallery,
          imageQuality: 85,
        );
        if (file != null && context.mounted) {
          Navigator.pop(context);
          onMediaSelected([file]);
        }
      }
    }
  }

  /// Show media picker as bottom sheet
  static Future<void> show(
    BuildContext context, {
    required Function(List<XFile>) onMediaSelected,
    bool allowMultiple = true,
    bool allowVideo = true,
  }) {
    return showModalBottomSheet(
      context: context,
      builder: (context) => MediaPicker(
        onMediaSelected: onMediaSelected,
        allowMultiple: allowMultiple,
        allowVideo: allowVideo,
      ),
    );
  }
}
