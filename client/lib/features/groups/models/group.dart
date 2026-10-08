/// 群组模型
class Group {
  final String id;
  final String name;
  final String? description;
  final String? avatarUrl;
  final String ownerId;
  final bool isPublic;
  final int memberCount;
  final int maxCapacity;
  final String status; // active, dissolved
  final DateTime createdAt;
  final DateTime updatedAt;

  Group({
    required this.id,
    required this.name,
    this.description,
    this.avatarUrl,
    required this.ownerId,
    this.isPublic = false,
    this.memberCount = 0,
    this.maxCapacity = 500,
    this.status = 'active',
    required this.createdAt,
    required this.updatedAt,
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

  /// 安全转换为 int
  static int _asInt(dynamic value, [int defaultValue = 0]) {
    if (value == null) return defaultValue;
    if (value is int) return value;
    if (value is double) return value.toInt();
    if (value is String) return int.tryParse(value) ?? defaultValue;
    return defaultValue;
  }

  /// 安全转换为 bool
  static bool _asBool(dynamic value, [bool defaultValue = false]) {
    if (value == null) return defaultValue;
    if (value is bool) return value;
    if (value is String) return value.toLowerCase() == 'true';
    if (value is num) return value != 0;
    return defaultValue;
  }

  /// 安全解析 DateTime
  static DateTime _asDateTime(dynamic value) {
    if (value == null) return DateTime.now();
    if (value is DateTime) return value;
    final str = _asString(value);
    if (str != null) {
      return DateTime.tryParse(str) ?? DateTime.now();
    }
    return DateTime.now();
  }

  factory Group.fromJson(Map<String, dynamic> json) {
    return Group(
      id: _asString(json['id']) ?? '',
      name: _asString(json['name']) ?? '',
      description: _asString(json['description']),
      // 后端返回 avatar 不是 avatar_url
      avatarUrl: _asString(json['avatar']) ?? _asString(json['avatar_url']),
      ownerId: _asString(json['owner_id']) ?? '',
      isPublic: _asBool(json['is_public']),
      memberCount: _asInt(json['member_count']),
      maxCapacity: _asInt(json['max_capacity'], 500),
      status: _asString(json['status']) ?? 'active',
      createdAt: _asDateTime(json['created_at']),
      updatedAt: json['updated_at'] != null
          ? _asDateTime(json['updated_at'])
          : _asDateTime(json['created_at']),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'description': description,
      'avatar': avatarUrl,
      'owner_id': ownerId,
      'is_public': isPublic,
      'member_count': memberCount,
      'max_capacity': maxCapacity,
      'status': status,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// 群组成员模型
/// 对应后端 GroupMemberInfo
class GroupMember {
  final String id; // user_id 作为 id
  final String username;
  final String? avatarUrl;
  final String role; // 'owner', 'admin', 'member'
  final bool isMuted;
  final bool isOnline;
  final DateTime joinedAt;

  GroupMember({
    required this.id,
    required this.username,
    this.avatarUrl,
    this.role = 'member',
    this.isMuted = false,
    this.isOnline = false,
    required this.joinedAt,
  });

  String get displayName => username;
  bool get isOwner => role.toLowerCase() == 'owner';
  bool get isAdmin => role.toLowerCase() == 'admin' || isOwner;

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

  /// 安全解析 DateTime
  static DateTime _asDateTime(dynamic value) {
    if (value == null) return DateTime.now();
    if (value is DateTime) return value;
    final str = _asString(value);
    if (str != null) {
      return DateTime.tryParse(str) ?? DateTime.now();
    }
    return DateTime.now();
  }

  factory GroupMember.fromJson(Map<String, dynamic> json) {
    // 后端 GroupMemberInfo 返回 user_id 而不是 id
    final userId = _asString(json['user_id']) ?? _asString(json['id']) ?? '';

    // role 可能是字符串或枚举对象
    String role = 'member';
    final rawRole = json['role'];
    if (rawRole is String) {
      role = rawRole.toLowerCase();
    } else if (rawRole is Map) {
      role = _asString(rawRole['type'])?.toLowerCase() ?? 'member';
    } else {
      role = _asString(rawRole)?.toLowerCase() ?? 'member';
    }

    return GroupMember(
      id: userId,
      username: _asString(json['username']) ?? '',
      avatarUrl: _asString(json['avatar_url']) ?? _asString(json['avatar']),
      role: role,
      isMuted: _asBool(json['is_muted']),
      isOnline: _asBool(json['online']) || _asBool(json['is_online']),
      joinedAt: _asDateTime(json['joined_at']),
    );
  }
}
