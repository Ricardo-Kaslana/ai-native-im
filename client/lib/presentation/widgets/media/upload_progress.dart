import 'package:flutter/material.dart';
import '../../../core/constants/spacing.dart';

/// Upload progress indicator
class UploadProgress extends StatelessWidget {
  final double progress;
  final String? fileName;
  final VoidCallback? onCancel;

  const UploadProgress({
    super.key,
    required this.progress,
    this.fileName,
    this.onCancel,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final percentage = (progress * 100).toInt();

    return Container(
      padding: const EdgeInsets.all(Spacing.md),
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        borderRadius: BorderRadius.circular(Spacing.radiusMd),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.1),
            blurRadius: 8,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Row(
        children: [
          // Progress circle
          SizedBox(
            width: 40,
            height: 40,
            child: Stack(
              alignment: Alignment.center,
              children: [
                CircularProgressIndicator(
                  value: progress,
                  strokeWidth: 3,
                  backgroundColor: theme.colorScheme.surfaceContainerHighest,
                ),
                Text(
                  '$percentage%',
                  style: theme.textTheme.labelSmall,
                ),
              ],
            ),
          ),
          const SizedBox(width: Spacing.md),

          // File info
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  fileName ?? '上传中...',
                  style: theme.textTheme.bodyMedium,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                const SizedBox(height: Spacing.xxs),
                LinearProgressIndicator(
                  value: progress,
                  backgroundColor: theme.colorScheme.surfaceContainerHighest,
                ),
              ],
            ),
          ),

          // Cancel button
          if (onCancel != null) ...[
            const SizedBox(width: Spacing.sm),
            IconButton(
              icon: const Icon(Icons.close),
              onPressed: onCancel,
              iconSize: 20,
            ),
          ],
        ],
      ),
    );
  }
}

/// Multiple uploads progress overlay
class UploadsProgressOverlay extends StatelessWidget {
  final List<UploadItem> uploads;
  final VoidCallback? onDismiss;

  const UploadsProgressOverlay({
    super.key,
    required this.uploads,
    this.onDismiss,
  });

  @override
  Widget build(BuildContext context) {
    if (uploads.isEmpty) return const SizedBox.shrink();

    final completedCount = uploads.where((u) => u.progress >= 1.0).length;
    final totalProgress = uploads.fold<double>(0, (sum, u) => sum + u.progress) / uploads.length;

    return Container(
      margin: const EdgeInsets.all(Spacing.md),
      padding: const EdgeInsets.all(Spacing.md),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(Spacing.radiusMd),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.15),
            blurRadius: 12,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            children: [
              const Icon(Icons.cloud_upload),
              const SizedBox(width: Spacing.sm),
              Expanded(
                child: Text(
                  '上传中 ($completedCount/${uploads.length})',
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
              ),
              if (onDismiss != null)
                IconButton(
                  icon: const Icon(Icons.close),
                  onPressed: onDismiss,
                  iconSize: 20,
                ),
            ],
          ),
          const SizedBox(height: Spacing.sm),
          LinearProgressIndicator(value: totalProgress),
        ],
      ),
    );
  }
}

/// Upload item model
class UploadItem {
  final String id;
  final String fileName;
  final double progress;
  final bool isError;

  const UploadItem({
    required this.id,
    required this.fileName,
    this.progress = 0,
    this.isError = false,
  });
}
