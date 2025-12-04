import 'package:flutter/material.dart';
import '../../../core/constants/spacing.dart';

/// Error view widget
class ErrorView extends StatelessWidget {
  final String? title;
  final String? message;
  final IconData icon;
  final VoidCallback? onRetry;
  final String retryText;

  const ErrorView({
    super.key,
    this.title,
    this.message,
    this.icon = Icons.error_outline,
    this.onRetry,
    this.retryText = '重试',
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Center(
      child: Padding(
        padding: const EdgeInsets.all(Spacing.lg),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              icon,
              size: 64,
              color: theme.colorScheme.error.withOpacity(0.7),
            ),
            const SizedBox(height: Spacing.lg),
            if (title != null)
              Text(
                title!,
                style: theme.textTheme.titleLarge,
                textAlign: TextAlign.center,
              ),
            if (message != null) ...[
              const SizedBox(height: Spacing.xs),
              Text(
                message!,
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: theme.textTheme.bodySmall?.color,
                ),
                textAlign: TextAlign.center,
              ),
            ],
            if (onRetry != null) ...[
              const SizedBox(height: Spacing.lg),
              ElevatedButton.icon(
                onPressed: onRetry,
                icon: const Icon(Icons.refresh),
                label: Text(retryText),
              ),
            ],
          ],
        ),
      ),
    );
  }

  /// Network error view
  factory ErrorView.network({VoidCallback? onRetry}) {
    return ErrorView(
      title: '网络连接失败',
      message: '请检查网络设置后重试',
      icon: Icons.wifi_off,
      onRetry: onRetry,
    );
  }

  /// Server error view
  factory ErrorView.server({VoidCallback? onRetry}) {
    return ErrorView(
      title: '服务器错误',
      message: '服务暂时不可用，请稍后重试',
      icon: Icons.cloud_off,
      onRetry: onRetry,
    );
  }

  /// Empty state view
  factory ErrorView.empty({
    String? title,
    String? message,
    IconData icon = Icons.inbox,
  }) {
    return ErrorView(
      title: title ?? '暂无数据',
      message: message,
      icon: icon,
    );
  }
}

/// Offline banner widget
class OfflineBanner extends StatelessWidget {
  final VoidCallback? onRetry;

  const OfflineBanner({super.key, this.onRetry});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(
        horizontal: Spacing.md,
        vertical: Spacing.sm,
      ),
      color: theme.colorScheme.errorContainer,
      child: Row(
        children: [
          Icon(
            Icons.wifi_off,
            size: 18,
            color: theme.colorScheme.onErrorContainer,
          ),
          const SizedBox(width: Spacing.sm),
          Expanded(
            child: Text(
              '网络连接已断开',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onErrorContainer,
              ),
            ),
          ),
          if (onRetry != null)
            TextButton(
              onPressed: onRetry,
              style: TextButton.styleFrom(
                foregroundColor: theme.colorScheme.onErrorContainer,
                padding: const EdgeInsets.symmetric(horizontal: Spacing.sm),
                minimumSize: Size.zero,
                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
              ),
              child: const Text('重试'),
            ),
        ],
      ),
    );
  }
}

/// Snackbar helper
class AppSnackBar {
  static void showError(BuildContext context, String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Row(
          children: [
            const Icon(Icons.error_outline, color: Colors.white),
            const SizedBox(width: Spacing.sm),
            Expanded(child: Text(message)),
          ],
        ),
        backgroundColor: Theme.of(context).colorScheme.error,
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  static void showSuccess(BuildContext context, String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Row(
          children: [
            const Icon(Icons.check_circle_outline, color: Colors.white),
            const SizedBox(width: Spacing.sm),
            Expanded(child: Text(message)),
          ],
        ),
        backgroundColor: Colors.green,
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  static void showInfo(BuildContext context, String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }
}
