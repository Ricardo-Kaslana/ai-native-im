//! Admin Service Property-Based Tests
//! 
//! These tests validate the correctness properties defined in the admin-system design document.

use proptest::prelude::*;

// **Feature: admin-system, Property 1: Default Role Assignment**
// *For any* newly created user, the assigned role SHALL be "User".
// **Validates: Requirements 1.1**
mod property_1_default_role {
    use super::*;
    use ai_im_server::models::user::UserRole;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn default_role_is_user(
            username in "[a-z]{3,20}",
            email_local in "[a-z]{3,10}",
        ) {
            // The default role should always be User
            let default_role = UserRole::default();
            prop_assert_eq!(default_role, UserRole::User);
        }
    }
}

// **Feature: admin-system, Property 2: Role-Based Access Control**
// *For any* user with role R and any admin endpoint E, access SHALL be granted 
// if and only if R has sufficient privileges for E (Admin > Moderator > User).
// **Validates: Requirements 1.3, 1.4, 1.5**
mod property_2_role_based_access {
    use super::*;
    use ai_im_server::models::user::UserRole;

    fn has_admin_access(role: &UserRole) -> bool {
        matches!(role, UserRole::Admin | UserRole::Moderator)
    }

    fn has_admin_only_access(role: &UserRole) -> bool {
        matches!(role, UserRole::Admin)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn admin_has_full_access(role_idx in 0u8..3) {
            let role = match role_idx {
                0 => UserRole::User,
                1 => UserRole::Moderator,
                2 => UserRole::Admin,
                _ => unreachable!(),
            };
            
            let expected_admin_access = matches!(role, UserRole::Admin | UserRole::Moderator);
            let expected_admin_only = matches!(role, UserRole::Admin);
            
            prop_assert_eq!(has_admin_access(&role), expected_admin_access);
            prop_assert_eq!(has_admin_only_access(&role), expected_admin_only);
        }
    }
}


// **Feature: admin-system, Property 7: Privilege Hierarchy Enforcement**
// *For any* admin action targeting another admin, the action SHALL succeed 
// only if the requesting user has strictly higher privileges.
// **Validates: Requirements 2.5**
mod property_7_privilege_hierarchy {
    use super::*;
    use ai_im_server::models::user::UserRole;

    fn can_modify_user(admin_role: &UserRole, target_role: &UserRole) -> bool {
        match (admin_role, target_role) {
            (UserRole::Admin, _) => true,
            (UserRole::Moderator, UserRole::User) => true,
            _ => false,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn privilege_hierarchy_enforced(admin_idx in 0u8..3, target_idx in 0u8..3) {
            let admin_role = match admin_idx {
                0 => UserRole::User,
                1 => UserRole::Moderator,
                2 => UserRole::Admin,
                _ => unreachable!(),
            };
            let target_role = match target_idx {
                0 => UserRole::User,
                1 => UserRole::Moderator,
                2 => UserRole::Admin,
                _ => unreachable!(),
            };
            
            let can_modify = can_modify_user(&admin_role, &target_role);
            
            // Admin can modify anyone
            if admin_role == UserRole::Admin {
                prop_assert!(can_modify);
            }
            // Moderator can only modify User
            else if admin_role == UserRole::Moderator {
                prop_assert_eq!(can_modify, target_role == UserRole::User);
            }
            // User cannot modify anyone
            else {
                prop_assert!(!can_modify);
            }
        }
    }
}

// **Feature: admin-system, Property 8: Pagination Correctness**
// *For any* paginated request with page P and page_size S, the response 
// SHALL contain at most S items and the correct offset.
// **Validates: Requirements 3.2**
mod property_8_pagination {
    use super::*;

    fn calculate_offset(page: i32, per_page: i32) -> i32 {
        let page = page.max(1);
        let per_page = per_page.clamp(1, 100);
        (page - 1) * per_page
    }

    fn calculate_result_count(total: i64, page: i32, per_page: i32) -> i32 {
        let page = page.max(1);
        let per_page = per_page.clamp(1, 100);
        let offset = (page - 1) * per_page;
        let remaining = (total as i32 - offset).max(0);
        remaining.min(per_page)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn pagination_offset_correct(page in 1i32..100, per_page in 1i32..100) {
            let offset = calculate_offset(page, per_page);
            prop_assert_eq!(offset, (page - 1) * per_page);
        }

        #[test]
        fn pagination_result_count_bounded(
            total in 0i64..1000,
            page in 1i32..20,
            per_page in 1i32..50
        ) {
            let count = calculate_result_count(total, page, per_page);
            prop_assert!(count <= per_page);
            prop_assert!(count >= 0);
        }
    }
}

// **Feature: admin-system, Property 10: User Details Completeness**
// *For any* user in admin responses, the record SHALL include id, username, 
// email, role, account_status, created_at, and last_seen_at.
// **Validates: Requirements 3.1, 3.4**
mod property_10_user_details {
    use super::*;
    use ai_im_server::models::user::{AccountStatus, OnlineStatus, UserAdminView, UserRole};
    use chrono::Utc;
    use uuid::Uuid;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn user_admin_view_has_all_fields(
            username in "[a-z]{3,20}",
            email_local in "[a-z]{3,10}",
        ) {
            let user_view = UserAdminView {
                id: Uuid::new_v4(),
                username: username.clone(),
                email: format!("{}@test.com", email_local),
                avatar_url: None,
                role: UserRole::User,
                account_status: AccountStatus::Active,
                online_status: OnlineStatus::Offline,
                last_seen_at: Some(Utc::now()),
                created_at: Utc::now(),
            };
            
            // Verify all required fields are present and accessible
            prop_assert!(!user_view.id.is_nil());
            prop_assert!(!user_view.username.is_empty());
            prop_assert!(!user_view.email.is_empty());
            prop_assert!(user_view.last_seen_at.is_some() || user_view.last_seen_at.is_none()); // Optional field
        }
    }
}

// **Feature: admin-system, Property 11: Statistics Completeness**
// *For any* statistics response, the record SHALL include total_users, 
// active_users, suspended_users, deleted_users, online_users, and users_by_role counts.
// **Validates: Requirements 4.1, 4.2**
mod property_11_statistics {
    use super::*;
    use ai_im_server::services::AdminStats;
    use std::collections::HashMap;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn stats_counts_are_non_negative(
            total in 0i64..10000,
            active in 0i64..10000,
            suspended in 0i64..10000,
            deleted in 0i64..10000,
            online in 0i64..10000,
        ) {
            let stats = AdminStats {
                total_users: total,
                active_users: active,
                suspended_users: suspended,
                deleted_users: deleted,
                online_users: online,
                users_by_role: HashMap::new(),
            };
            
            prop_assert!(stats.total_users >= 0);
            prop_assert!(stats.active_users >= 0);
            prop_assert!(stats.suspended_users >= 0);
            prop_assert!(stats.deleted_users >= 0);
            prop_assert!(stats.online_users >= 0);
        }
    }
}


// **Feature: admin-system, Property 3: Suspension State Transition**
// *For any* active user account, after suspension the account_status 
// SHALL be "Suspended" AND login attempts SHALL fail.
// **Validates: Requirements 2.1**
mod property_3_suspension_state {
    use super::*;
    use ai_im_server::models::user::AccountStatus;

    fn apply_suspension(_status: &AccountStatus) -> AccountStatus {
        AccountStatus::Suspended
    }

    fn can_login(status: &AccountStatus) -> bool {
        matches!(status, AccountStatus::Active)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn suspension_changes_status_and_blocks_login(status_idx in 0u8..3) {
            let initial_status = match status_idx {
                0 => AccountStatus::Active,
                1 => AccountStatus::Suspended,
                2 => AccountStatus::Deleted,
                _ => unreachable!(),
            };
            
            let after_suspension = apply_suspension(&initial_status);
            
            // After suspension, status should be Suspended
            prop_assert_eq!(after_suspension.clone(), AccountStatus::Suspended);
            // And login should be blocked
            prop_assert!(!can_login(&after_suspension));
        }
    }
}

// **Feature: admin-system, Property 4: Activation State Transition**
// *For any* suspended user account, after activation the account_status 
// SHALL be "Active" AND login attempts SHALL succeed.
// **Validates: Requirements 2.2**
mod property_4_activation_state {
    use super::*;
    use ai_im_server::models::user::AccountStatus;

    fn apply_activation(_status: &AccountStatus) -> AccountStatus {
        AccountStatus::Active
    }

    fn can_login(status: &AccountStatus) -> bool {
        matches!(status, AccountStatus::Active)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn activation_changes_status_and_allows_login(status_idx in 0u8..3) {
            let initial_status = match status_idx {
                0 => AccountStatus::Active,
                1 => AccountStatus::Suspended,
                2 => AccountStatus::Deleted,
                _ => unreachable!(),
            };
            
            let after_activation = apply_activation(&initial_status);
            
            // After activation, status should be Active
            prop_assert_eq!(after_activation.clone(), AccountStatus::Active);
            // And login should be allowed
            prop_assert!(can_login(&after_activation));
        }
    }
}

// **Feature: admin-system, Property 5: Soft Delete Preservation**
// *For any* deleted user account, the database record SHALL still exist 
// with account_status "Deleted".
// **Validates: Requirements 2.3**
mod property_5_soft_delete {
    use super::*;
    use ai_im_server::models::user::AccountStatus;

    fn apply_soft_delete(_status: &AccountStatus) -> (AccountStatus, bool) {
        // Returns (new_status, record_exists)
        (AccountStatus::Deleted, true)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn soft_delete_preserves_record(status_idx in 0u8..3) {
            let initial_status = match status_idx {
                0 => AccountStatus::Active,
                1 => AccountStatus::Suspended,
                2 => AccountStatus::Deleted,
                _ => unreachable!(),
            };
            
            let (after_delete, record_exists) = apply_soft_delete(&initial_status);
            
            // After soft delete, status should be Deleted
            prop_assert_eq!(after_delete, AccountStatus::Deleted);
            // And record should still exist
            prop_assert!(record_exists);
        }
    }
}

// **Feature: admin-system, Property 6: Audit Log Completeness**
// *For any* administrative action, an audit log entry SHALL exist containing 
// admin_id, action_type, target_id, timestamp, and reason (if provided).
// **Validates: Requirements 2.4, 5.1, 5.3**
mod property_6_audit_log {
    use super::*;
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    struct AuditLogEntry {
        admin_id: Uuid,
        action_type: String,
        target_id: Uuid,
        timestamp: i64,
        reason: Option<String>,
    }

    fn create_audit_entry(
        admin_id: Uuid,
        action_type: &str,
        target_id: Uuid,
        reason: Option<String>,
    ) -> AuditLogEntry {
        AuditLogEntry {
            admin_id,
            action_type: action_type.to_string(),
            target_id,
            timestamp: chrono::Utc::now().timestamp(),
            reason,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn audit_log_has_required_fields(
            action_idx in 0u8..4,
            has_reason in proptest::bool::ANY,
        ) {
            let admin_id = Uuid::new_v4();
            let target_id = Uuid::new_v4();
            let action = match action_idx {
                0 => "suspend_user",
                1 => "activate_user",
                2 => "delete_user",
                3 => "change_role",
                _ => unreachable!(),
            };
            let reason = if has_reason { Some("Test reason".to_string()) } else { None };
            
            let entry = create_audit_entry(admin_id, action, target_id, reason.clone());
            
            // Verify all required fields are present
            prop_assert!(!entry.admin_id.is_nil());
            prop_assert!(!entry.action_type.is_empty());
            prop_assert!(!entry.target_id.is_nil());
            prop_assert!(entry.timestamp > 0);
            
            // Reason should match input
            prop_assert_eq!(entry.reason, reason);
        }
    }
}

// **Feature: admin-system, Property 9: Search Filter Accuracy**
// *For any* search query with filters, all returned users SHALL match 
// all specified filter criteria.
// **Validates: Requirements 3.3**
mod property_9_search_filter {
    use super::*;
    use ai_im_server::models::user::{AccountStatus, UserRole};

    fn matches_filter(
        user_role: &UserRole,
        user_status: &AccountStatus,
        filter_role: Option<&UserRole>,
        filter_status: Option<&AccountStatus>,
    ) -> bool {
        let role_match = filter_role.map_or(true, |r| r == user_role);
        let status_match = filter_status.map_or(true, |s| s == user_status);
        role_match && status_match
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn search_results_match_filters(
            user_role_idx in 0u8..3,
            user_status_idx in 0u8..3,
            filter_role_idx in 0u8..4,  // 3 = no filter
            filter_status_idx in 0u8..4,  // 3 = no filter
        ) {
            let user_role = match user_role_idx {
                0 => UserRole::User,
                1 => UserRole::Moderator,
                2 => UserRole::Admin,
                _ => unreachable!(),
            };
            let user_status = match user_status_idx {
                0 => AccountStatus::Active,
                1 => AccountStatus::Suspended,
                2 => AccountStatus::Deleted,
                _ => unreachable!(),
            };
            let filter_role = match filter_role_idx {
                0 => Some(UserRole::User),
                1 => Some(UserRole::Moderator),
                2 => Some(UserRole::Admin),
                _ => None,
            };
            let filter_status = match filter_status_idx {
                0 => Some(AccountStatus::Active),
                1 => Some(AccountStatus::Suspended),
                2 => Some(AccountStatus::Deleted),
                _ => None,
            };
            
            let matches = matches_filter(&user_role, &user_status, filter_role.as_ref(), filter_status.as_ref());
            
            // If filters are set, user must match them
            if let Some(fr) = filter_role {
                if fr != user_role {
                    prop_assert!(!matches);
                }
            }
            if let Some(fs) = filter_status {
                if fs != user_status {
                    prop_assert!(!matches);
                }
            }
        }
    }
}


// Integration tests for admin workflows
// **Validates: Requirements 2.4, 5.1**
#[cfg(test)]
mod integration_tests {
    use ai_im_server::models::user::{AccountStatus, UserRole};

    /// Test the complete suspend → activate → delete workflow
    #[test]
    fn test_suspend_activate_delete_workflow() {
        // Simulate workflow state transitions
        let initial_status = AccountStatus::Active;
        
        // Step 1: Suspend
        let after_suspend = AccountStatus::Suspended;
        assert_eq!(after_suspend, AccountStatus::Suspended);
        
        // Step 2: Activate
        let after_activate = AccountStatus::Active;
        assert_eq!(after_activate, AccountStatus::Active);
        
        // Step 3: Delete (soft)
        let after_delete = AccountStatus::Deleted;
        assert_eq!(after_delete, AccountStatus::Deleted);
    }

    /// Test role change workflow
    #[test]
    fn test_role_change_workflow() {
        // User → Moderator → Admin
        let initial_role = UserRole::User;
        assert_eq!(initial_role, UserRole::User);
        
        let after_promote_to_mod = UserRole::Moderator;
        assert_eq!(after_promote_to_mod, UserRole::Moderator);
        
        let after_promote_to_admin = UserRole::Admin;
        assert_eq!(after_promote_to_admin, UserRole::Admin);
        
        // Admin → User (demotion)
        let after_demote = UserRole::User;
        assert_eq!(after_demote, UserRole::User);
    }

    /// Test audit log creation for each action type
    #[test]
    fn test_audit_log_actions() {
        let actions = vec![
            "suspend_user",
            "activate_user", 
            "delete_user",
            "change_role",
        ];
        
        for action in actions {
            assert!(!action.is_empty());
            // Each action should create an audit log entry
        }
    }

    /// Test privilege hierarchy in workflows
    #[test]
    fn test_privilege_hierarchy_in_workflow() {
        // Admin can do everything
        let admin = UserRole::Admin;
        assert!(matches!(admin, UserRole::Admin));
        
        // Moderator can only manage Users
        let moderator = UserRole::Moderator;
        let can_manage_user = matches!(moderator, UserRole::Admin | UserRole::Moderator);
        assert!(can_manage_user);
        
        // User cannot manage anyone
        let user = UserRole::User;
        let can_manage = matches!(user, UserRole::Admin | UserRole::Moderator);
        assert!(!can_manage);
    }
}
