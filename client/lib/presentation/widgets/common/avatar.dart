import 'package:flutter/material.dart';
import 'package:cached_network_image/cached_network_image.dart';
import '../../../core/constants/colors.dart';

/// Custom avatar widget with placeholder support
class AppAvatar extends StatelessWidget {
  final String? imageUrl;
  final String? name;
  final double size;
  final bool isGroup;
  final bool isAI;

  const AppAvatar({
    super.key,
    this.imageUrl,
    this.name,
    this.size = 48,
    this.isGroup = false,
    this.isAI = false,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;

    // Get initials from name
    String initials = '?';
    if (name != null && name!.isNotEmpty) {
      final parts = name!.trim().split(' ');
      if (parts.length >= 2) {
        initials = '${parts[0][0]}${parts[1][0]}'.toUpperCase();
      } else {
        initials = name!.substring(0, name!.length >= 2 ? 2 : 1).toUpperCase();
      }
    }

    // Generate color from name
    final backgroundColor = _getColorFromName(name ?? '', isDark);

    Widget child;

    if (isAI) {
      // AI agent avatar
      child = Container(
        width: size,
        height: size,
        decoration: BoxDecoration(
          gradient: LinearGradient(
            colors: [
              theme.colorScheme.primary,
              theme.colorScheme.primary.withOpacity(0.7),
            ],
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
          ),
          borderRadius: BorderRadius.circular(size / 4),
        ),
        child: Icon(
          Icons.smart_toy,
          color: Colors.white,
          size: size * 0.5,
        ),
      );
    } else if (imageUrl != null && imageUrl!.isNotEmpty) {
      // Network image avatar
      child = ClipRRect(
        borderRadius: BorderRadius.circular(isGroup ? size / 4 : size / 2),
        child: CachedNetworkImage(
          imageUrl: imageUrl!,
          width: size,
          height: size,
          fit: BoxFit.cover,
          placeholder: (context, url) => _buildPlaceholder(
            initials,
            backgroundColor,
            isGroup,
          ),
          errorWidget: (context, url, error) => _buildPlaceholder(
            initials,
            backgroundColor,
            isGroup,
          ),
        ),
      );
    } else {
      // Placeholder avatar with initials
      child = _buildPlaceholder(initials, backgroundColor, isGroup);
    }

    return child;
  }

  Widget _buildPlaceholder(String initials, Color backgroundColor, bool isGroup) {
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        color: backgroundColor,
        borderRadius: BorderRadius.circular(isGroup ? size / 4 : size / 2),
      ),
      child: Center(
        child: Text(
          initials,
          style: TextStyle(
            color: Colors.white,
            fontSize: size * 0.4,
            fontWeight: FontWeight.w600,
          ),
        ),
      ),
    );
  }

  Color _getColorFromName(String name, bool isDark) {
    if (name.isEmpty) {
      return isDark ? AppColors.darkPrimary : AppColors.lightPrimary;
    }

    // Generate a consistent color based on name hash
    final colors = [
      const Color(0xFF2196F3), // Blue
      const Color(0xFF4CAF50), // Green
      const Color(0xFFFF9800), // Orange
      const Color(0xFF9C27B0), // Purple
      const Color(0xFFE91E63), // Pink
      const Color(0xFF00BCD4), // Cyan
      const Color(0xFF795548), // Brown
      const Color(0xFF607D8B), // Blue Grey
    ];

    final hash = name.codeUnits.fold(0, (prev, curr) => prev + curr);
    return colors[hash % colors.length];
  }
}
