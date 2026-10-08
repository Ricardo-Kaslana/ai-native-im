import 'package:flutter/material.dart';
import '../../../core/utils/url_utils.dart';

/// 用户头像组件
class UserAvatar extends StatefulWidget {
  final String name;
  final String? avatarUrl;
  final double size;
  final VoidCallback? onTap;

  const UserAvatar({
    super.key,
    required this.name,
    this.avatarUrl,
    this.size = 40,
    this.onTap,
  });

  @override
  State<UserAvatar> createState() => _UserAvatarState();
}

class _UserAvatarState extends State<UserAvatar> {
  bool _imageLoadFailed = false;

  @override
  void didUpdateWidget(UserAvatar oldWidget) {
    super.didUpdateWidget(oldWidget);
    // 当 avatarUrl 变化时重置加载失败状态
    if (oldWidget.avatarUrl != widget.avatarUrl) {
      setState(() {
        _imageLoadFailed = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    Widget avatar;

    // 只有当 avatarUrl 不为空且图像加载未失败时才使用网络图像
    final hasAvatar = widget.avatarUrl != null && widget.avatarUrl!.isNotEmpty;

    if (hasAvatar && !_imageLoadFailed) {
      avatar = CircleAvatar(
        radius: widget.size / 2,
        backgroundImage: NetworkImage(UrlUtils.processMediaUrl(widget.avatarUrl!)),
        onBackgroundImageError: (error, stackTrace) {
          // 图像加载失败时回退到占位符
          if (mounted) {
            setState(() {
              _imageLoadFailed = true;
            });
          }
          debugPrint('UserAvatar: Failed to load image: ${widget.avatarUrl}, error: $error');
        },
        child: null,
      );
    } else {
      // 使用占位符
      if (hasAvatar && _imageLoadFailed) {
        debugPrint('UserAvatar: Using placeholder due to load failure for: ${widget.avatarUrl}');
      } else {
        debugPrint('UserAvatar: No avatar URL provided for: ${widget.name}');
      }

      // 生成基于名称的颜色
      final colorIndex = widget.name.isNotEmpty ? widget.name.codeUnitAt(0) % _colors.length : 0;
      final color = _colors[colorIndex];

      avatar = CircleAvatar(
        radius: widget.size / 2,
        backgroundColor: color,
        child: Text(
          _getInitials(widget.name),
          style: TextStyle(
            color: Colors.white,
            fontSize: widget.size * 0.4,
            fontWeight: FontWeight.w600,
          ),
        ),
      );
    }

    if (widget.onTap != null) {
      return GestureDetector(onTap: widget.onTap, child: avatar);
    }
    return avatar;
  }

  String _getInitials(String name) {
    if (name.isEmpty) return '?';
    final parts = name.trim().split(' ');
    if (parts.length >= 2) {
      return '${parts[0][0]}${parts[1][0]}'.toUpperCase();
    }
    return name[0].toUpperCase();
  }

  static const _colors = [
    Color(0xFF2563EB), // Blue
    Color(0xFF7C3AED), // Violet
    Color(0xFFDB2777), // Pink
    Color(0xFFDC2626), // Red
    Color(0xFFEA580C), // Orange
    Color(0xFF65A30D), // Lime
    Color(0xFF059669), // Emerald
    Color(0xFF0891B2), // Cyan
  ];
}
