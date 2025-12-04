import 'package:flutter/material.dart';
import '../../../data/models/message.dart';

/// Message status icon widget
class MessageStatusIcon extends StatelessWidget {
  final MessageStatus status;
  final Color? color;
  final double size;

  const MessageStatusIcon({
    super.key,
    required this.status,
    this.color,
    this.size = 14,
  });

  @override
  Widget build(BuildContext context) {
    final iconColor = color ?? Theme.of(context).textTheme.bodySmall?.color;

    switch (status) {
      case MessageStatus.sending:
        return SizedBox(
          width: size,
          height: size,
          child: CircularProgressIndicator(
            strokeWidth: 1.5,
            valueColor: AlwaysStoppedAnimation<Color>(iconColor!),
          ),
        );
      case MessageStatus.sent:
        return Icon(
          Icons.check,
          size: size,
          color: iconColor,
        );
      case MessageStatus.delivered:
        return Icon(
          Icons.done_all,
          size: size,
          color: iconColor,
        );
      case MessageStatus.read:
        return Icon(
          Icons.done_all,
          size: size,
          color: Colors.blue,
        );
      case MessageStatus.failed:
        return Icon(
          Icons.error_outline,
          size: size,
          color: Colors.red,
        );
    }
  }
}
