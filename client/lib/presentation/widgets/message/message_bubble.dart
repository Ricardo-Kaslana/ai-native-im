import 'package:flutter/material.dart';
import '../../../core/constants/colors.dart';
import '../../../core/constants/spacing.dart';
import '../../../data/models/message.dart';
import '../common/avatar.dart';
import 'message_status_icon.dart';

/// Message bubble widget
class MessageBubble extends StatelessWidget {
  final Message message;
  final bool isSent;
  final bool showAvatar;

  const MessageBubble({
    super.key,
    required this.message,
    required this.isSent,
    this.showAvatar = false,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;

    final bubbleColor = isSent
        ? (isDark ? AppColors.sentBubbleDark : AppColors.sentBubbleLight)
        : (isDark ? AppColors.receivedBubbleDark : AppColors.receivedBubbleLight);

    final textColor = isSent
        ? Colors.white
        : (isDark ? AppColors.darkText : AppColors.lightText);

    return Padding(
      padding: EdgeInsets.only(
        left: isSent ? Spacing.xxl : Spacing.md,
        right: isSent ? Spacing.md : Spacing.xxl,
        top: Spacing.xxs,
        bottom: Spacing.xxs,
      ),
      child: Row(
        mainAxisAlignment: isSent ? MainAxisAlignment.end : MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          // Avatar (for received messages in groups)
          if (!isSent && showAvatar) ...[
            AppAvatar(
              name: message.senderName,
              size: 28,
            ),
            const SizedBox(width: Spacing.xs),
          ] else if (!isSent && !showAvatar) ...[
            const SizedBox(width: 28 + Spacing.xs),
          ],

          // Bubble
          Flexible(
            child: Container(
              padding: const EdgeInsets.symmetric(
                horizontal: Spacing.md,
                vertical: Spacing.sm,
              ),
              decoration: BoxDecoration(
                color: bubbleColor,
                borderRadius: BorderRadius.only(
                  topLeft: const Radius.circular(Spacing.radiusLg),
                  topRight: const Radius.circular(Spacing.radiusLg),
                  bottomLeft: Radius.circular(isSent ? Spacing.radiusLg : Spacing.xxs),
                  bottomRight: Radius.circular(isSent ? Spacing.xxs : Spacing.radiusLg),
                ),
                boxShadow: [
                  BoxShadow(
                    color: Colors.black.withOpacity(0.05),
                    blurRadius: 4,
                    offset: const Offset(0, 2),
                  ),
                ],
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Sender name (for group messages)
                  if (!isSent && showAvatar && message.senderName != null) ...[
                    Text(
                      message.senderName!,
                      style: theme.textTheme.labelSmall?.copyWith(
                        color: theme.colorScheme.primary,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                    const SizedBox(height: Spacing.xxs),
                  ],

                  // Message content
                  _buildContent(context, textColor),

                  // Time and status
                  const SizedBox(height: Spacing.xxs),
                  Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        _formatTime(message.createdAt),
                        style: theme.textTheme.labelSmall?.copyWith(
                          color: isSent
                              ? Colors.white.withOpacity(0.7)
                              : theme.textTheme.bodySmall?.color,
                          fontSize: 10,
                        ),
                      ),
                      if (isSent) ...[
                        const SizedBox(width: Spacing.xxs),
                        MessageStatusIcon(
                          status: message.status,
                          color: Colors.white.withOpacity(0.7),
                        ),
                      ],
                    ],
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildContent(BuildContext context, Color textColor) {
    switch (message.type) {
      case MessageType.text:
        return Text(
          message.content,
          style: TextStyle(
            color: textColor,
            fontSize: 15,
            height: 1.4,
          ),
        );
      case MessageType.image:
        return _buildImageContent(context);
      case MessageType.video:
        return _buildVideoContent(context);
      case MessageType.audio:
        return _buildAudioContent(context, textColor);
      case MessageType.file:
        return _buildFileContent(context, textColor);
      case MessageType.system:
        return Text(
          message.content,
          style: TextStyle(
            color: textColor.withOpacity(0.8),
            fontSize: 13,
            fontStyle: FontStyle.italic,
          ),
        );
    }
  }

  Widget _buildImageContent(BuildContext context) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(Spacing.radiusSm),
      child: Container(
        constraints: const BoxConstraints(
          maxWidth: 200,
          maxHeight: 200,
        ),
        color: Colors.grey[300],
        child: const Center(
          child: Icon(Icons.image, size: 48, color: Colors.grey),
        ),
      ),
    );
  }

  Widget _buildVideoContent(BuildContext context) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(Spacing.radiusSm),
      child: Container(
        constraints: const BoxConstraints(
          maxWidth: 200,
          maxHeight: 200,
        ),
        color: Colors.grey[800],
        child: const Center(
          child: Icon(Icons.play_circle_fill, size: 48, color: Colors.white),
        ),
      ),
    );
  }

  Widget _buildAudioContent(BuildContext context, Color textColor) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(Icons.play_arrow, color: textColor),
        const SizedBox(width: Spacing.xs),
        Container(
          width: 100,
          height: 24,
          decoration: BoxDecoration(
            color: textColor.withOpacity(0.2),
            borderRadius: BorderRadius.circular(12),
          ),
        ),
        const SizedBox(width: Spacing.xs),
        Text(
          '0:15',
          style: TextStyle(color: textColor, fontSize: 12),
        ),
      ],
    );
  }

  Widget _buildFileContent(BuildContext context, Color textColor) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(Icons.insert_drive_file, color: textColor),
        const SizedBox(width: Spacing.xs),
        Flexible(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'document.pdf',
                style: TextStyle(color: textColor, fontWeight: FontWeight.w500),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              Text(
                '2.5 MB',
                style: TextStyle(
                  color: textColor.withOpacity(0.7),
                  fontSize: 12,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  String _formatTime(DateTime dateTime) {
    return '${dateTime.hour.toString().padLeft(2, '0')}:${dateTime.minute.toString().padLeft(2, '0')}';
  }
}
