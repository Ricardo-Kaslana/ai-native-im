use ai_im_server::api::{ConnectionManager, WsState};
use ai_im_server::services::presence::PresenceService;
use tokio::sync::broadcast;
use uuid::Uuid;

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// **Feature: user-auth-system, Property 53: WebSocket closure on logout**
    /// **Validates: Requirements 12.5**
    /// *For any* user logout, all active WebSocket connections for that user should be closed.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_websocket_closure_on_logout(
            connection_count in 1usize..5,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Create connection manager
                let connection_manager = ConnectionManager::new();
                let user_id = Uuid::new_v4();
                
                // Register multiple connections for the user
                let mut receivers = Vec::new();
                for _ in 0..connection_count {
                    let rx = connection_manager.register_connection(user_id).await;
                    receivers.push(rx);
                }
                
                // Verify connections are registered
                let count_before = connection_manager.connection_count(&user_id).await;
                prop_assert_eq!(count_before, connection_count);
                
                // Simulate logout - close all connections
                connection_manager.close_user_connections(&user_id).await;
                
                // Verify all receivers get the shutdown signal
                for mut rx in receivers {
                    let result = rx.try_recv();
                    prop_assert!(result.is_ok(), "Connection should receive shutdown signal");
                }
                
                // Verify connections are cleaned up
                let count_after = connection_manager.connection_count(&user_id).await;
                prop_assert_eq!(count_after, 0, "All connections should be closed after logout");
                
                Ok(())
            })?;
        }
    }

    /// Test that multiple users' connections are independent
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_user_connection_isolation(
            user1_connections in 1usize..3,
            user2_connections in 1usize..3,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let connection_manager = ConnectionManager::new();
                let user1 = Uuid::new_v4();
                let user2 = Uuid::new_v4();
                
                // Register connections for user1
                let mut user1_receivers = Vec::new();
                for _ in 0..user1_connections {
                    let rx = connection_manager.register_connection(user1).await;
                    user1_receivers.push(rx);
                }
                
                // Register connections for user2
                let mut user2_receivers = Vec::new();
                for _ in 0..user2_connections {
                    let rx = connection_manager.register_connection(user2).await;
                    user2_receivers.push(rx);
                }
                
                // Verify initial counts
                let user1_count_before = connection_manager.connection_count(&user1).await;
                let user2_count_before = connection_manager.connection_count(&user2).await;
                prop_assert_eq!(user1_count_before, user1_connections);
                prop_assert_eq!(user2_count_before, user2_connections);
                
                // Close user1's connections
                connection_manager.close_user_connections(&user1).await;
                
                // Verify user1's connections are closed
                for mut rx in user1_receivers {
                    let result = rx.try_recv();
                    prop_assert!(result.is_ok(), "User1 connections should be closed");
                }
                
                // Verify user2's connections are still open (no shutdown signal)
                for mut rx in &mut user2_receivers {
                    let result = rx.try_recv();
                    prop_assert!(result.is_err(), "User2 connections should remain open");
                }
                
                // Verify user1 count is 0
                let user1_count = connection_manager.connection_count(&user1).await;
                prop_assert_eq!(user1_count, 0);
                
                // User2's connections should still be counted (receivers are still alive)
                let user2_count = connection_manager.connection_count(&user2).await;
                prop_assert_eq!(user2_count, user2_connections, 
                    "User2 should still have {} connections, but has {}", 
                    user2_connections, user2_count);
                
                Ok(())
            })?;
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use ai_im_server::models::user::OnlineStatus;

    #[tokio::test]
    async fn test_status_update_on_connection() {
        // Test that connecting updates user status to Online
        let presence_service = PresenceService::new();
        let user_id = Uuid::new_v4();
        
        // Connect user
        let update = presence_service.user_connected(user_id).await.unwrap();
        
        // Verify status is Online
        assert_eq!(update.status, OnlineStatus::Online);
        assert_eq!(update.user_id, user_id);
        
        // Verify presence is stored
        let presence = presence_service.get_presence(&user_id).await.unwrap();
        assert_eq!(presence.status, OnlineStatus::Online);
        assert_eq!(presence.user_id, user_id);
    }

    #[tokio::test]
    async fn test_status_update_on_disconnection() {
        // Test that disconnecting updates user status to Offline
        let presence_service = PresenceService::new();
        let user_id = Uuid::new_v4();
        
        // Connect user first
        presence_service.user_connected(user_id).await.unwrap();
        
        // Disconnect user
        let update = presence_service.user_disconnected(user_id).await.unwrap();
        
        // Verify status is Offline
        assert!(update.is_some());
        let update = update.unwrap();
        assert_eq!(update.status, OnlineStatus::Offline);
        assert_eq!(update.user_id, user_id);
        
        // Verify presence is updated
        let presence = presence_service.get_presence(&user_id).await.unwrap();
        assert_eq!(presence.status, OnlineStatus::Offline);
    }

    #[tokio::test]
    async fn test_connection_closure_on_logout() {
        // Test that logout closes all WebSocket connections
        let connection_manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        
        // Simulate multiple active connections
        let mut rx1 = connection_manager.register_connection(user_id).await;
        let mut rx2 = connection_manager.register_connection(user_id).await;
        let mut rx3 = connection_manager.register_connection(user_id).await;
        
        // Verify connections are active
        assert_eq!(connection_manager.connection_count(&user_id).await, 3);
        
        // Simulate logout - close all connections
        connection_manager.close_user_connections(&user_id).await;
        
        // Verify all connections receive shutdown signal
        assert!(rx1.try_recv().is_ok(), "Connection 1 should be closed");
        assert!(rx2.try_recv().is_ok(), "Connection 2 should be closed");
        assert!(rx3.try_recv().is_ok(), "Connection 3 should be closed");
        
        // Verify connection count is 0
        assert_eq!(connection_manager.connection_count(&user_id).await, 0);
    }

    #[tokio::test]
    async fn test_multiple_connections_single_user() {
        // Test that a user can have multiple simultaneous connections
        let presence_service = PresenceService::new();
        let connection_manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        
        // Connect multiple times (e.g., mobile + desktop)
        presence_service.user_connected(user_id).await.unwrap();
        presence_service.user_connected(user_id).await.unwrap();
        
        let _rx1 = connection_manager.register_connection(user_id).await;
        let _rx2 = connection_manager.register_connection(user_id).await;
        
        // Verify user is online
        let presence = presence_service.get_presence(&user_id).await.unwrap();
        assert_eq!(presence.status, OnlineStatus::Online);
        
        // Verify multiple connections are tracked
        assert_eq!(connection_manager.connection_count(&user_id).await, 2);
        
        // Disconnect one connection
        presence_service.user_disconnected(user_id).await.unwrap();
        
        // User should still be online (one connection remains)
        let presence = presence_service.get_presence(&user_id).await.unwrap();
        assert_eq!(presence.status, OnlineStatus::Online);
    }

    #[tokio::test]
    async fn test_connection_registration_and_closure() {
        let connection_manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        
        // Register a connection
        let mut rx = connection_manager.register_connection(user_id).await;
        
        // Verify connection is registered
        let count = connection_manager.connection_count(&user_id).await;
        assert_eq!(count, 1);
        
        // Close connections
        connection_manager.close_user_connections(&user_id).await;
        
        // Verify shutdown signal is received
        let result = rx.try_recv();
        assert!(result.is_ok(), "Should receive shutdown signal");
        
        // Verify connection count is 0
        let count = connection_manager.connection_count(&user_id).await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_multiple_connections_same_user() {
        let connection_manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        
        // Register 3 connections
        let mut rx1 = connection_manager.register_connection(user_id).await;
        let mut rx2 = connection_manager.register_connection(user_id).await;
        let mut rx3 = connection_manager.register_connection(user_id).await;
        
        // Verify all connections are registered
        let count = connection_manager.connection_count(&user_id).await;
        assert_eq!(count, 3);
        
        // Close all connections
        connection_manager.close_user_connections(&user_id).await;
        
        // Verify all receive shutdown signal
        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
        assert!(rx3.try_recv().is_ok());
        
        // Verify all connections are closed
        let count = connection_manager.connection_count(&user_id).await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_connection_unregister() {
        let connection_manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        
        // Register a connection
        let _rx = connection_manager.register_connection(user_id).await;
        
        // Verify connection is registered
        let count = connection_manager.connection_count(&user_id).await;
        assert_eq!(count, 1);
        
        // Unregister connection (simulating normal disconnect)
        connection_manager.unregister_connection(&user_id).await;
        
        // Note: unregister only removes closed channels, so count may still be 1
        // unless the receiver is dropped
        drop(_rx);
        
        // After dropping the receiver, unregister should clean it up
        connection_manager.unregister_connection(&user_id).await;
        let count = connection_manager.connection_count(&user_id).await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_multiple_users_isolation() {
        let connection_manager = ConnectionManager::new();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        
        // Register connections for both users
        let mut user1_rx = connection_manager.register_connection(user1).await;
        let mut user2_rx = connection_manager.register_connection(user2).await;
        
        // Close user1's connections
        connection_manager.close_user_connections(&user1).await;
        
        // Verify user1's connection is closed
        assert!(user1_rx.try_recv().is_ok());
        
        // Verify user2's connection is still open
        assert!(user2_rx.try_recv().is_err());
        
        // Verify counts
        let user1_count = connection_manager.connection_count(&user1).await;
        let user2_count = connection_manager.connection_count(&user2).await;
        assert_eq!(user1_count, 0);
        assert_eq!(user2_count, 1);
    }
}
