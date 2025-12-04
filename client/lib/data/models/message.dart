import 'package:equatable/equatable.dart';

/// Message model
class Message extends Equatable {
  final String id;
  final String conversationId;
  final String senderId;
  final String? senderName;
  final String? senderAvatar;
  final MessageType type;
  final String content;
  final MessageStatus status;
  final String? replyToId;
  final List<MediaAttachment>? attachments;
  final DateTime createdAt;
  final DateTime? editedAt;
  final bool isDeleted;

  const Message({
    required this.id,
    required this.conversationId,
    required this.senderId,
    this.senderName,
    this.senderAvatar,
    required this.type,
    required this.content,
    this.status = MessageStatus.sending,
    this.replyToId,
    this.attachments,
    required this.createdAt,
    this.editedAt,
    this.isDeleted = false,
  });

  /// Get preview text for conversation list
  String get preview {
    if (isDeleted) return '消息已删除';
    switch (type) {
      case MessageType.text:
        return content;
      case MessageType.image:
        return '[图片]';
      case MessageType.video:
        return '[视频]';
      case MessageType.audio:
        return '[语音]';
      case MessageType.file:
        return '[文件]';
      case MessageType.system:
        return content;
    }
  }

  Message copyWith({
    String? id,
    String? conversationId,
    String? senderId,
    String? senderName,
    String? senderAvatar,
    MessageType? type,
    String? content,
    MessageStatus? status,
    String? replyToId,
    List<MediaAttachment>? attachments,
    DateTime? createdAt,
    DateTime? editedAt,
    bool? isDeleted,
  }) {
    return Message(
      id: id ?? this.id,
      conversationId: conversationId ?? this.conversationId,
      senderId: senderId ?? this.senderId,
      senderName: senderName ?? this.senderName,
      senderAvatar: senderAvatar ?? this.senderAvatar,
      type: type ?? this.type,
      content: content ?? this.content,
      status: status ?? this.status,
      replyToId: replyToId ?? this.replyToId,
      attachments: attachments ?? this.attachments,
      createdAt: createdAt ?? this.createdAt,
      editedAt: editedAt ?? this.editedAt,
      isDeleted: isDeleted ?? this.isDeleted,
    );
  }

  @override
  List<Object?> get props => [
        id,
        conversationId,
        senderId,
        senderName,
        senderAvatar,
        type,
        content,
        status,
        replyToId,
        attachments,
        createdAt,
        editedAt,
        isDeleted,
      ];
}

/// Message type
enum MessageType {
  text,
  image,
  video,
  audio,
  file,
  system,
}

/// Message status
enum MessageStatus {
  sending,
  sent,
  delivered,
  read,
  failed,
}

/// Media attachment
class MediaAttachment extends Equatable {
  final String id;
  final String url;
  final String? thumbnailUrl;
  final MediaType type;
  final String? fileName;
  final int? fileSize;
  final int? width;
  final int? height;
  final int? duration;

  const MediaAttachment({
    required this.id,
    required this.url,
    this.thumbnailUrl,
    required this.type,
    this.fileName,
    this.fileSize,
    this.width,
    this.height,
    this.duration,
  });

  @override
  List<Object?> get props => [
        id,
        url,
        thumbnailUrl,
        type,
        fileName,
        fileSize,
        width,
        height,
        duration,
      ];
}

/// Media type
enum MediaType {
  image,
  video,
  audio,
  file,
}
