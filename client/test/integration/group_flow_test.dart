import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:client/presentation/screens/groups/group_info_screen.dart';
import 'package:client/data/models/group.dart';
import 'package:client/data/models/user.dart';

void main() {
  group('Group Flow Integration Test', () {
    testWidgets('group model has correct structure', (tester) async {
      // Test group model
      final group = Group(
        id: 'group-1',
        name: 'Test Group',
        description: 'A test group for testing',
        avatarUrl: 'https://example.com/group.jpg',
        memberCount: 10,
        ownerId: 'owner-1',
        isPublic: true,
        createdAt: DateTime.now(),
      );

      expect(group.id, 'group-1');
      expect(group.name, 'Test Group');
      expect(group.memberCount, 10);
      expect(group.isPublic, true);
    });

    testWidgets('private group model works correctly', (tester) async {
      // Test private group
      final privateGroup = Group(
        id: 'group-2',
        name: 'Private Group',
        description: 'A private group',
        avatarUrl: null,
        memberCount: 5,
        ownerId: 'owner-2',
        isPublic: false,
        createdAt: DateTime.now(),
      );

      expect(privateGroup.isPublic, false);
      expect(privateGroup.avatarUrl, null);
    });

    testWidgets('group info screen displays correctly', (tester) async {
      final testGroup = Group(
        id: 'test-group',
        name: 'Test Group',
        description: 'Test description',
        ownerId: 'owner-1',
        memberCount: 5,
        createdAt: DateTime.now(),
      );

      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: GroupInfoScreen(group: testGroup),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Verify group info screen is displayed
      expect(find.byType(GroupInfoScreen), findsOneWidget);
      expect(find.text('Test Group'), findsOneWidget);
    });

    testWidgets('group member model has correct structure', (tester) async {
      // Test group member model with actual structure
      final testUser = User(
        id: 'user-1',
        username: 'testuser',
        email: 'test@example.com',
        createdAt: DateTime.now(),
      );

      final member = GroupMember(
        id: 'member-1',
        groupId: 'group-1',
        user: testUser,
        role: GroupRole.admin,
        joinedAt: DateTime.now(),
      );

      expect(member.id, 'member-1');
      expect(member.user.username, 'testuser');
      expect(member.role, GroupRole.admin);
      expect(member.isAdmin, true);
    });

    testWidgets('group roles are correctly defined', (tester) async {
      // Test group roles
      expect(GroupRole.owner.index, isNot(GroupRole.admin.index));
      expect(GroupRole.admin.index, isNot(GroupRole.member.index));
      expect(GroupRole.owner.index, isNot(GroupRole.member.index));
    });

    testWidgets('group member isOwner and isAdmin work correctly', (tester) async {
      final testUser = User(
        id: 'user-1',
        username: 'owner',
        email: 'owner@example.com',
        createdAt: DateTime.now(),
      );

      final ownerMember = GroupMember(
        id: 'member-1',
        groupId: 'group-1',
        user: testUser,
        role: GroupRole.owner,
        joinedAt: DateTime.now(),
      );

      expect(ownerMember.isOwner, true);
      expect(ownerMember.isAdmin, true); // Owner is also admin

      final regularMember = GroupMember(
        id: 'member-2',
        groupId: 'group-1',
        user: testUser,
        role: GroupRole.member,
        joinedAt: DateTime.now(),
      );

      expect(regularMember.isOwner, false);
      expect(regularMember.isAdmin, false);
    });
  });
}
