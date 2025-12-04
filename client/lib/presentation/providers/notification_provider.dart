import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Notification settings state
class NotificationSettings {
  final bool enabled;
  final bool soundEnabled;
  final bool vibrationEnabled;
  final bool showPreview;
  final List<String> mutedConversations;

  const NotificationSettings({
    this.enabled = true,
    this.soundEnabled = true,
    this.vibrationEnabled = true,
    this.showPreview = true,
    this.mutedConversations = const [],
  });

  NotificationSettings copyWith({
    bool? enabled,
    bool? soundEnabled,
    bool? vibrationEnabled,
    bool? showPreview,
    List<String>? mutedConversations,
  }) {
    return NotificationSettings(
      enabled: enabled ?? this.enabled,
      soundEnabled: soundEnabled ?? this.soundEnabled,
      vibrationEnabled: vibrationEnabled ?? this.vibrationEnabled,
      showPreview: showPreview ?? this.showPreview,
      mutedConversations: mutedConversations ?? this.mutedConversations,
    );
  }

  bool isConversationMuted(String conversationId) {
    return mutedConversations.contains(conversationId);
  }
}

/// Notification settings notifier
class NotificationSettingsNotifier extends StateNotifier<NotificationSettings> {
  NotificationSettingsNotifier() : super(const NotificationSettings());

  void setEnabled(bool enabled) {
    state = state.copyWith(enabled: enabled);
  }

  void setSoundEnabled(bool enabled) {
    state = state.copyWith(soundEnabled: enabled);
  }

  void setVibrationEnabled(bool enabled) {
    state = state.copyWith(vibrationEnabled: enabled);
  }

  void setShowPreview(bool show) {
    state = state.copyWith(showPreview: show);
  }

  void muteConversation(String conversationId) {
    if (!state.mutedConversations.contains(conversationId)) {
      state = state.copyWith(
        mutedConversations: [...state.mutedConversations, conversationId],
      );
    }
  }

  void unmuteConversation(String conversationId) {
    state = state.copyWith(
      mutedConversations: state.mutedConversations
          .where((id) => id != conversationId)
          .toList(),
    );
  }
}

/// Notification settings provider
final notificationSettingsProvider =
    StateNotifierProvider<NotificationSettingsNotifier, NotificationSettings>(
  (ref) => NotificationSettingsNotifier(),
);

/// In-app notification model
class InAppNotification {
  final String id;
  final String title;
  final String? body;
  final String? imageUrl;
  final String? conversationId;
  final DateTime createdAt;
  final NotificationType type;

  const InAppNotification({
    required this.id,
    required this.title,
    this.body,
    this.imageUrl,
    this.conversationId,
    required this.createdAt,
    this.type = NotificationType.message,
  });
}

/// Notification type
enum NotificationType {
  message,
  mention,
  groupInvite,
  friendRequest,
  system,
}

/// In-app notifications provider
final inAppNotificationsProvider = StateProvider<List<InAppNotification>>((ref) => []);

/// Show in-app notification
void showInAppNotification(WidgetRef ref, InAppNotification notification) {
  final settings = ref.read(notificationSettingsProvider);
  if (!settings.enabled) return;
  if (notification.conversationId != null &&
      settings.isConversationMuted(notification.conversationId!)) {
    return;
  }

  final notifications = ref.read(inAppNotificationsProvider);
  ref.read(inAppNotificationsProvider.notifier).state = [
    notification,
    ...notifications.take(4), // Keep max 5 notifications
  ];

  // Auto dismiss after 4 seconds
  Future.delayed(const Duration(seconds: 4), () {
    final current = ref.read(inAppNotificationsProvider);
    ref.read(inAppNotificationsProvider.notifier).state =
        current.where((n) => n.id != notification.id).toList();
  });
}
