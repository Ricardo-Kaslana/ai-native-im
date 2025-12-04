import 'package:equatable/equatable.dart';
import 'user.dart';

/// Group model
class Group extends Equatable {
  final String id;
  final String name;
  final String? description;
  final String? avatarUrl;
  final String ownerId;
  final int memberCount;
  final int maxMembers;
  final bool isPublic;
  final String? inviteCode;
  final DateTime createdAt;
  final DateTime? updatedAt;

  const Group({
    required this.id,
    required this.name,
    this.description,
    this.avatarUrl,
    required this.ownerId,
    this.memberCount = 1,
    this.maxMembers = 500,
    this.isPublic = false,
    this.inviteCode,
    required this.createdAt,
    this.updatedAt,
  });

  Group copyWith({
    String? id,
    String? name,
    String? description,
    String? avatarUrl,
    String? ownerId,
    int? memberCount,
    int? maxMembers,
    bool? isPublic,
    String? inviteCode,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) {
    return Group(
      id: id ?? this.id,
      name: name ?? this.name,
      description: description ?? this.description,
      avatarUrl: avatarUrl ?? this.avatarUrl,
      ownerId: ownerId ?? this.ownerId,
      memberCount: memberCount ?? this.memberCount,
      maxMembers: maxMembers ?? this.maxMembers,
      isPublic: isPublic ?? this.isPublic,
      inviteCode: inviteCode ?? this.inviteCode,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }

  @override
  List<Object?> get props => [
        id,
        name,
        description,
        avatarUrl,
        ownerId,
        memberCount,
        maxMembers,
        isPublic,
        inviteCode,
        createdAt,
        updatedAt,
      ];
}

/// Group member model
class GroupMember extends Equatable {
  final String id;
  final String groupId;
  final User user;
  final GroupRole role;
  final bool isMuted;
  final DateTime? mutedUntil;
  final DateTime joinedAt;

  const GroupMember({
    required this.id,
    required this.groupId,
    required this.user,
    this.role = GroupRole.member,
    this.isMuted = false,
    this.mutedUntil,
    required this.joinedAt,
  });

  bool get isOwner => role == GroupRole.owner;
  bool get isAdmin => role == GroupRole.admin || role == GroupRole.owner;

  @override
  List<Object?> get props => [id, groupId, user, role, isMuted, mutedUntil, joinedAt];
}

/// Group role
enum GroupRole {
  owner,
  admin,
  member,
}
