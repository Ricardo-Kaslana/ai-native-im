import 'package:flutter/material.dart';

/// AI Agent 专用头像组件
/// 使用机器人图标和渐变色背景，便于区分
class AiAgentAvatar extends StatelessWidget {
  final String? agentName;
  final double size;
  final VoidCallback? onTap;

  const AiAgentAvatar({
    super.key,
    this.agentName,
    this.size = 40,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final avatar = Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        gradient: const LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            Color(0xFF6366F1), // Indigo 500
            Color(0xFF8B5CF6), // Violet 500
            Color(0xFFEC4899), // Pink 500
          ],
        ),
        shape: BoxShape.circle,
        border: Border.all(
          color: Colors.white.withValues(alpha: 0.3),
          width: 2,
        ),
      ),
      child: Center(
        child: agentName != null && agentName!.isNotEmpty
            ? Text(
                agentName![0].toUpperCase(),
                style: TextStyle(
                  color: Colors.white,
                  fontSize: size * 0.4,
                  fontWeight: FontWeight.bold,
                ),
              )
            : Icon(
                Icons.smart_toy_outlined,
                size: size * 0.5,
                color: Colors.white,
              ),
      ),
    );

    if (onTap != null) {
      return GestureDetector(onTap: onTap, child: avatar);
    }
    return avatar;
  }
}
