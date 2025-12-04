import 'package:equatable/equatable.dart';

/// User model
class User extends Equatable {
  final String id;
  final String username;
  final String? email;
  final String? avatarUrl;
  final String? bio;
  final bool isOnline;
  final DateTime? lastSeenAt;
  final DateTime createdAt;

  const User({
    required this.id,
    required this.username,
    this.email,
    this.avatarUrl,
    this.bio,
    this.isOnline = false,
    this.lastSeenAt,
    required this.createdAt,
  });

  /// Get display name
  String get displayName => username;

  /// Get initials for avatar placeholder
  String get initials {
    if (username.isEmpty) return '?';
    final parts = username.split(' ');
    if (parts.length >= 2) {
      return '${parts[0][0]}${parts[1][0]}'.toUpperCase();
    }
    return username.substring(0, username.length >= 2 ? 2 : 1).toUpperCase();
  }

  User copyWith({
    String? id,
    String? username,
    String? email,
    String? avatarUrl,
    String? bio,
    bool? isOnline,
    DateTime? lastSeenAt,
    DateTime? createdAt,
  }) {
    return User(
      id: id ?? this.id,
      username: username ?? this.username,
      email: email ?? this.email,
      avatarUrl: avatarUrl ?? this.avatarUrl,
      bio: bio ?? this.bio,
      isOnline: isOnline ?? this.isOnline,
      lastSeenAt: lastSeenAt ?? this.lastSeenAt,
      createdAt: createdAt ?? this.createdAt,
    );
  }

  @override
  List<Object?> get props => [
        id,
        username,
        email,
        avatarUrl,
        bio,
        isOnline,
        lastSeenAt,
        createdAt,
      ];
}
