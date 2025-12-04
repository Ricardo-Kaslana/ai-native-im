import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/models/group.dart';
import '../../data/models/user.dart';

/// Groups provider
final groupsProvider = FutureProvider<List<Group>>((ref) async {
  await Future.delayed(const Duration(milliseconds: 300));
  return _mockGroups;
});

/// Group members provider
final groupMembersProvider = FutureProvider.family<List<GroupMember>, String>((ref, groupId) async {
  await Future.delayed(const Duration(milliseconds: 200));
  return _mockMembers.where((m) => m.groupId == groupId).toList();
});

/// Selected members for group creation
final selectedMembersProvider = StateProvider<List<User>>((ref) => []);

/// Group creation state
class GroupCreationState {
  final String name;
  final String? description;
  final bool isPublic;
  final bool isLoading;
  final String? error;

  const GroupCreationState({
    this.name = '',
    this.description,
    this.isPublic = false,
    this.isLoading = false,
    this.error,
  });

  GroupCreationState copyWith({
    String? name,
    String? description,
    bool? isPublic,
    bool? isLoading,
    String? error,
  }) {
    return GroupCreationState(
      name: name ?? this.name,
      description: description ?? this.description,
      isPublic: isPublic ?? this.isPublic,
      isLoading: isLoading ?? this.isLoading,
      error: error,
    );
  }
}

/// Group creation notifier
class GroupCreationNotifier extends StateNotifier<GroupCreationState> {
  GroupCreationNotifier() : super(const GroupCreationState());

  void setName(String name) => state = state.copyWith(name: name);
  void setDescription(String? desc) => state = state.copyWith(description: desc);
  void setPublic(bool isPublic) => state = state.copyWith(isPublic: isPublic);

  Future<Group?> createGroup(List<User> members) async {
    if (state.name.trim().isEmpty) {
      state = state.copyWith(error: '请输入群组名称');
      return null;
    }

    state = state.copyWith(isLoading: true, error: null);

    try {
      await Future.delayed(const Duration(seconds: 1));

      final group = Group(
        id: 'group_${DateTime.now().millisecondsSinceEpoch}',
        name: state.name.trim(),
        description: state.description,
        ownerId: 'me',
        memberCount: members.length + 1,
        isPublic: state.isPublic,
        createdAt: DateTime.now(),
      );

      state = state.copyWith(isLoading: false);
      return group;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
      return null;
    }
  }

  void reset() => state = const GroupCreationState();
}

final groupCreationProvider = StateNotifierProvider<GroupCreationNotifier, GroupCreationState>(
  (ref) => GroupCreationNotifier(),
);

/// Mock data
final _mockGroups = [
  Group(
    id: 'group_1',
    name: '项目讨论组',
    description: '讨论项目相关事宜',
    ownerId: 'me',
    memberCount: 5,
    isPublic: false,
    inviteCode: 'ABC123',
    createdAt: DateTime.now().subtract(const Duration(days: 14)),
  ),
  Group(
    id: 'group_2',
    name: 'Flutter 学习群',
    description: '一起学习 Flutter 开发',
    ownerId: 'user_1',
    memberCount: 28,
    isPublic: true,
    createdAt: DateTime.now().subtract(const Duration(days: 30)),
  ),
];

final _mockMembers = [
  GroupMember(
    id: 'member_1',
    groupId: 'group_1',
    user: User(id: 'me', username: '我', createdAt: DateTime.now()),
    role: GroupRole.owner,
    joinedAt: DateTime.now().subtract(const Duration(days: 14)),
  ),
  GroupMember(
    id: 'member_2',
    groupId: 'group_1',
    user: User(id: 'user_1', username: '张三', isOnline: true, createdAt: DateTime.now()),
    role: GroupRole.admin,
    joinedAt: DateTime.now().subtract(const Duration(days: 10)),
  ),
  GroupMember(
    id: 'member_3',
    groupId: 'group_1',
    user: User(id: 'user_2', username: '李四', createdAt: DateTime.now()),
    role: GroupRole.member,
    joinedAt: DateTime.now().subtract(const Duration(days: 7)),
  ),
];
