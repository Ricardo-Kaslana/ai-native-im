import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../../../core/constants/spacing.dart';
import '../../../core/constants/typography.dart';

/// Primary button with loading state
class AppButton extends StatelessWidget {
  final String text;
  final VoidCallback? onPressed;
  final bool isLoading;
  final bool isOutlined;
  final IconData? icon;
  final double? width;

  const AppButton({
    super.key,
    required this.text,
    this.onPressed,
    this.isLoading = false,
    this.isOutlined = false,
    this.icon,
    this.width,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    Widget child = isLoading
        ? SizedBox(
            height: 20,
            width: 20,
            child: CircularProgressIndicator(
              strokeWidth: 2,
              valueColor: AlwaysStoppedAnimation<Color>(
                isOutlined ? theme.colorScheme.primary : Colors.white,
              ),
            ),
          )
        : Row(
            mainAxisSize: MainAxisSize.min,
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              if (icon != null) ...[
                Icon(icon, size: Spacing.iconMd),
                const SizedBox(width: Spacing.xs),
              ],
              Text(text, style: AppTypography.button),
            ],
          );

    final button = isOutlined
        ? OutlinedButton(
            onPressed: isLoading ? null : _handlePress,
            style: OutlinedButton.styleFrom(
              padding: const EdgeInsets.symmetric(
                horizontal: Spacing.lg,
                vertical: Spacing.md,
              ),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(Spacing.radiusMd),
              ),
              side: BorderSide(color: theme.colorScheme.primary),
            ),
            child: child,
          )
        : ElevatedButton(
            onPressed: isLoading ? null : _handlePress,
            child: child,
          );

    return width != null
        ? SizedBox(width: width, child: button)
        : button;
  }

  void _handlePress() {
    HapticFeedback.lightImpact();
    onPressed?.call();
  }
}

/// Text button variant
class AppTextButton extends StatelessWidget {
  final String text;
  final VoidCallback? onPressed;
  final IconData? icon;

  const AppTextButton({
    super.key,
    required this.text,
    this.onPressed,
    this.icon,
  });

  @override
  Widget build(BuildContext context) {
    return TextButton(
      onPressed: () {
        HapticFeedback.selectionClick();
        onPressed?.call();
      },
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (icon != null) ...[
            Icon(icon, size: Spacing.iconSm),
            const SizedBox(width: Spacing.xxs),
          ],
          Text(text),
        ],
      ),
    );
  }
}
