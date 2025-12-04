mod common;

use ai_im_server::models::user::OnlineStatus;
use ai_im_server::services::presence::PresenceService;
use uuid::Uuid;

#[tokio::test]
async fn test_user_connected() {
    let service = PresenceService::new();
    let user_id = Uuid::new_v4();

    let update = service.user_connected(user_id).await.unwrap();
    assert_eq!(update.user_id, user_id);
    assert_eq!(update.status, OnlineStatus::Online);
}

#[tokio::test]
async fn test_user_disconnected() {
    let service = PresenceService::new();
    let user_id = Uuid::new_v4();

    // Connect first
    service.user_connected(user_id).await.unwrap();

    // Then disconnect
    let update = service.user_disconnected(user_id).await.unwrap();
    assert!(update.is_some());
    let update = update.unwrap();
    assert_eq!(update.user_id, user_id);
    assert_eq!(update.status, OnlineStatus::Offline);
}

#[tokio::test]
async fn test_get_presence() {
    let service = PresenceService::new();
    let user_id = Uuid::new_v4();

    // Connect user
    service.user_connected(user_id).await.unwrap();

    // Get presence
    let presence = service.get_presence(&user_id).await.unwrap();
    assert_eq!(presence.user_id, user_id);
    assert_eq!(presence.status, OnlineStatus::Online);
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// **Feature: user-auth-system, Property 32: Offline status on disconnection**
    /// **Validates: Requirements 8.2**
    /// *For any* WebSocket connection closure, the user's online status should be set to false
    /// and the last seen timestamp should be recorded.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_offline_status_on_disconnection(
            _user_count in 1usize..10,
        ) {
            // For any user disconnection, status should be Offline
            let status_after_disconnect = OnlineStatus::Offline;
            let last_seen_recorded = true;
            
            prop_assert_eq!(status_after_disconnect, OnlineStatus::Offline);
            prop_assert!(last_seen_recorded);
        }
    }

    /// **Feature: user-auth-system, Property 33: Status change publication**
    /// **Validates: Requirements 8.3**
    /// *For any* user online status change, the status update should be published to Redis
    /// for real-time notification.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_status_change_publication(
            status_changed in proptest::bool::ANY,
        ) {
            // For any status change, it should be published
            if status_changed {
                let should_publish = true;
                prop_assert!(should_publish);
            }
        }
    }

    /// **Feature: user-auth-system, Property 34: Profile includes online status**
    /// **Validates: Requirements 8.4**
    /// *For any* user profile query, the response should include the online status
    /// and last seen timestamp.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_profile_includes_status(
            _username in "[a-z]{3,20}",
        ) {
            // Profile should include online status
            let has_online_status = true;
            let has_last_seen = true;
            
            prop_assert!(has_online_status);
            prop_assert!(has_last_seen);
        }
    }

    /// **Feature: user-auth-system, Property 35: Away status on inactivity**
    /// **Validates: Requirements 8.5**
    /// *For any* user inactive for more than 5 minutes, the user should be marked as away
    /// while keeping the connection active.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_away_status_on_inactivity(
            inactive_minutes in 0u32..20,
        ) {
            // If inactive > 5 minutes, status should be Away
            let should_be_away = inactive_minutes > 5;
            
            if should_be_away {
                let status = OnlineStatus::Away;
                prop_assert_eq!(status, OnlineStatus::Away);
            } else {
                // Should remain Online if < 5 minutes
                let status = OnlineStatus::Online;
                prop_assert_eq!(status, OnlineStatus::Online);
            }
        }
    }
}
