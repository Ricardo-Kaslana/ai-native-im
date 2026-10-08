/// 用户模型
class User {
  final String id;
  final String username;
  final String? email;
  final String? avatarUrl;
  final String? bio;
  final String onlineStatus; // offline, online, away
  final String role;
  final DateTime createdAt;
  final DateTime? updatedAt;

  User({
    required this.id,
    required this.username,
    this.email,
    this.avatarUrl,
    this.bio,
    this.onlineStatus = 'offline',
    this.role = 'user',
    required this.createdAt,
    this.updatedAt,
  });

  /// 安全转换为 String
  static String? _asString(dynamic value) {
    if (value == null) return null;
    if (value is String) return value;
    // 处理数字类型（int, double, num）
    if (value is num) return value.toString();
    // 其他类型转字符串
    return value.toString();
  }

  factory User.fromJson(Map<String, dynamic> json) {
    return User(
      id: _asString(json['id']) ?? '',
      username: _asString(json['username']) ?? '',
      email: _asString(json['email']),
      avatarUrl: _asString(json['avatar_url']),
      bio: _asString(json['bio']),
      onlineStatus: _asString(json['online_status']) ?? 'offline',
      role: _asString(json['role']) ?? 'user',
      createdAt: json['created_at'] != null
          ? DateTime.parse(_asString(json['created_at'])!)
          : DateTime.now(),
      updatedAt: json['updated_at'] != null
          ? DateTime.parse(_asString(json['updated_at'])!)
          : null,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'username': username,
      'email': email,
      'avatar_url': avatarUrl,
      'bio': bio,
      'online_status': onlineStatus,
      'role': role,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt?.toIso8601String(),
    };
  }

  /// 显示名称
  String get displayName => username;

  /// 是否在线
  bool get isOnline => onlineStatus == 'online';

  User copyWith({
    String? id,
    String? username,
    String? email,
    String? avatarUrl,
    String? bio,
    String? onlineStatus,
    String? role,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) {
    return User(
      id: id ?? this.id,
      username: username ?? this.username,
      email: email ?? this.email,
      avatarUrl: avatarUrl ?? this.avatarUrl,
      bio: bio ?? this.bio,
      onlineStatus: onlineStatus ?? this.onlineStatus,
      role: role ?? this.role,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }
}
