/// 联系人模型
/// API 返回的联系人实际上是用户信息
class Contact {
  final String id;
  final String username;
  final String? email;
  final String? avatarUrl;
  final String? bio;
  final bool isOnline;
  final DateTime createdAt;

  Contact({
    required this.id,
    required this.username,
    this.email,
    this.avatarUrl,
    this.bio,
    this.isOnline = false,
    required this.createdAt,
  });

  /// 显示名称
  String get displayName => username;

  /// 安全转换为 String
  static String? _asString(dynamic value) {
    if (value == null) return null;
    if (value is String) return value;
    // 处理数字类型（int, double, num）
    if (value is num) return value.toString();
    // 其他类型转字符串
    return value.toString();
  }

  /// 安全转换为 bool
  static bool _asBool(dynamic value, [bool defaultValue = false]) {
    if (value == null) return defaultValue;
    if (value is bool) return value;
    if (value is String) return value.toLowerCase() == 'true';
    if (value is num) return value != 0;
    return defaultValue;
  }

  factory Contact.fromJson(Map<String, dynamic> json) {
    return Contact(
      id: _asString(json['id']) ?? '',
      username: _asString(json['username']) ?? '',
      email: _asString(json['email']),
      avatarUrl: _asString(json['avatar_url']),
      bio: _asString(json['bio']),
      isOnline: _asBool(json['is_online']),
      createdAt: json['created_at'] != null
          ? DateTime.parse(_asString(json['created_at'])!)
          : DateTime.now(),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'username': username,
      'email': email,
      'avatar_url': avatarUrl,
      'bio': bio,
      'is_online': isOnline,
      'created_at': createdAt.toIso8601String(),
    };
  }
}
