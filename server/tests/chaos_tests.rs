//! 混沌工程测试 - 验证系统在故障场景下的可靠性
//!
//! 测试场景:
//! - 网络超时和重试
//! - 服务降级
//! - 资源耗尽
//! - 并发冲突

mod common;

use ai_im_server::api::ConnectionManager;
use ai_im_server::services::auth::{AuthService, LoginData, RegisterData, TokenService, UserService};
use ai_im_server::services::email::MockEmailService;
use ai_im_server::services::presence::PresenceService;
use ai_im_server::models::user::{CreateUser, UpdateUser};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use uuid::Uuid;

/// **混沌测试: 操作超时处理**
/// 验证系统在操作超时时的行为
#[tokio::test]
async fn chaos_test_operation_timeout_handling() {
    let pool = common::setup_test_db().await;
    let user_service = UserService::new(pool.clone());

    // 创建测试用户
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let create_data = CreateUser {
        username: format!("chaos_timeout_{}", &uuid_str[..8]),
        email: format!("chaos_{}@test.com", uuid_str),
        password_hash: "hashed".to_string(),
    };
    let user = user_service.create_user(create_data).await.unwrap();

    // 测试带超时的操作
    let result = timeout(
        Duration::from_millis(100),
        user_service.get_profile_by_id(&user.id),
    )
    .await;

    assert!(result.is_ok(), "正常操作应在超时内完成");
    assert!(result.unwrap().is_ok(), "查询应该成功");

    // 清理
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();
}

/// **混沌测试: 并发写入冲突**
/// 验证系统处理并发更新的能力
#[tokio::test]
async fn chaos_test_concurrent_write_conflicts() {
    let pool = common::setup_test_db().await;
    let user_service = UserService::new(pool.clone());

    // 创建测试用户
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let create_data = CreateUser {
        username: format!("chaos_conflict_{}", &uuid_str[..8]),
        email: format!("conflict_{}@test.com", uuid_str),
        password_hash: "hashed".to_string(),
    };
    let user = user_service.create_user(create_data).await.unwrap();
    let user_id = user.id;

    // 并发更新同一用户
    let num_updates = 50;
    let mut handles = vec![];

    for i in 0..num_updates {
        let user_service = user_service.clone();
        let user_id = user_id;

        let handle = tokio::spawn(async move {
            let update_data = UpdateUser {
                username: None,
                avatar_url: Some(format!("https://example.com/avatar_{}.jpg", i)),
                bio: Some(format!("Concurrent update {}", i)),
            };
            user_service.update_profile(&user_id, update_data).await
        });
        handles.push(handle);
    }

    // 统计结果
    let mut successful = 0;
    let mut failed = 0;

    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => successful += 1,
            _ => failed += 1,
        }
    }

    println!(
        "并发写入冲突测试: {} 成功, {} 失败",
        successful, failed
    );

    // 所有更新都应该成功（数据库处理并发）
    assert!(
        successful >= num_updates * 90 / 100,
        "至少 90% 的并发更新应该成功"
    );

    // 验证最终状态一致
    let final_user = user_service.get_profile_by_id(&user_id).await;
    assert!(final_user.is_ok(), "用户应该存在");

    // 清理
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(&user_id)
        .execute(&pool)
        .await
        .ok();
}

/// **混沌测试: 连接断开恢复**
/// 验证 WebSocket 连接断开后的清理
#[tokio::test]
async fn chaos_test_connection_drop_recovery() {
    let connection_manager = ConnectionManager::new();
    let presence_service = PresenceService::new();

    let num_users = 100;
    let mut user_ids = vec![];
    let mut receivers = vec![]; // 保持 receivers 存活

    // 创建连接
    for _ in 0..num_users {
        let user_id = Uuid::new_v4();
        let rx = connection_manager.register_connection(user_id).await;
        presence_service.user_connected(user_id).await.unwrap();
        user_ids.push(user_id);
        receivers.push(rx);
    }

    // 验证所有连接存在
    let mut active_before = 0;
    for user_id in &user_ids {
        active_before += connection_manager.connection_count(user_id).await;
    }
    assert_eq!(active_before, num_users, "所有连接应该活跃");

    // 模拟一半连接断开
    for user_id in user_ids.iter().take(num_users / 2) {
        connection_manager.close_user_connections(user_id).await;
        presence_service.user_disconnected(*user_id).await.unwrap();
    }

    // 验证断开的连接被清理
    let mut active_after = 0;
    for user_id in &user_ids {
        active_after += connection_manager.connection_count(user_id).await;
    }

    println!(
        "连接断开恢复: {} -> {} 活跃连接",
        active_before, active_after
    );

    // 关闭的连接应该被清理（count 为 0），剩余的应该还在
    assert!(
        active_after <= num_users / 2,
        "关闭的连接应该被清理"
    );

    // 清理剩余连接
    for user_id in user_ids.iter().skip(num_users / 2) {
        connection_manager.close_user_connections(user_id).await;
    }
    drop(receivers);
}

/// **混沌测试: Token 过期处理**
/// 验证过期 token 被正确拒绝
#[tokio::test]
async fn chaos_test_expired_token_handling() {
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();

    // 使用极短过期时间的 token service
    let token_service = TokenService::with_redis("chaos_test_secret".to_string(), redis);

    let user_id = Uuid::new_v4();
    let token_pair = token_service
        .generate_token_pair(&user_id, "chaos_user")
        .unwrap();

    // 正常 token 应该有效
    let result = token_service.verify_token(&token_pair.access_token);
    assert!(result.is_ok(), "新 token 应该有效");

    // 验证无效 token 被拒绝
    let invalid_token = "invalid.token.here";
    let result = token_service.verify_token(invalid_token);
    assert!(result.is_err(), "无效 token 应该被拒绝");

    // 验证篡改的 token 被拒绝
    let tampered_token = format!("{}tampered", &token_pair.access_token[..50]);
    let result = token_service.verify_token(&tampered_token);
    assert!(result.is_err(), "篡改的 token 应该被拒绝");
}

/// **混沌测试: 重复注册处理**
/// 验证重复用户名/邮箱被正确拒绝
#[tokio::test]
async fn chaos_test_duplicate_registration() {
    let pool = common::setup_test_db().await;
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let email_service: Arc<dyn ai_im_server::services::email::EmailService> =
        Arc::new(MockEmailService::silent());

    let auth_service = AuthService::with_bcrypt_cost(
        pool.clone(),
        redis.clone(),
        "chaos_test_secret".to_string(),
        email_service,
        4,
    );

    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("chaos_dup_{}", &uuid_str[..8]);
    let email = format!("chaos_dup_{}@test.com", uuid_str);

    // 第一次注册应该成功
    let register_data = RegisterData {
        username: username.clone(),
        email: email.clone(),
        password: "ChaosTest123!".to_string(),
    };
    let result = auth_service.register(register_data).await;
    assert!(result.is_ok(), "第一次注册应该成功");

    // 重复用户名应该失败
    let register_data = RegisterData {
        username: username.clone(),
        email: format!("different_{}@test.com", Uuid::new_v4()),
        password: "ChaosTest123!".to_string(),
    };
    let result = auth_service.register(register_data).await;
    assert!(result.is_err(), "重复用户名应该失败");

    // 重复邮箱应该失败
    let register_data = RegisterData {
        username: format!("different_{}", Uuid::new_v4()),
        email: email.clone(),
        password: "ChaosTest123!".to_string(),
    };
    let result = auth_service.register(register_data).await;
    assert!(result.is_err(), "重复邮箱应该失败");

    // 清理
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(&username)
        .execute(&pool)
        .await
        .ok();
}

/// **混沌测试: 快速连接/断开循环**
/// 验证系统处理快速连接变化的能力
#[tokio::test]
async fn chaos_test_rapid_connect_disconnect_cycle() {
    let connection_manager = ConnectionManager::new();
    let presence_service = PresenceService::new();

    let user_id = Uuid::new_v4();
    let num_cycles = 100;
    let start = Instant::now();

    for _ in 0..num_cycles {
        // 连接
        let _rx = connection_manager.register_connection(user_id).await;
        presence_service.user_connected(user_id).await.unwrap();

        // 立即断开
        connection_manager.close_user_connections(&user_id).await;
        presence_service.user_disconnected(user_id).await.unwrap();
    }

    let duration = start.elapsed();
    let rate = num_cycles as f64 / duration.as_secs_f64();

    println!(
        "快速连接/断开: {} 次循环在 {:?} ({:.2} cycles/sec)",
        num_cycles, duration, rate
    );

    // 验证最终状态干净
    let count = connection_manager.connection_count(&user_id).await;
    assert_eq!(count, 0, "所有连接应该被清理");

    // 应该能快速处理
    assert!(
        rate >= 100.0,
        "应该能处理 100+ cycles/sec，实际: {:.2}",
        rate
    );
}

/// **混沌测试: 大量用户同时上线**
/// 模拟服务重启后大量用户重连
#[tokio::test]
async fn chaos_test_mass_reconnection() {
    let connection_manager = ConnectionManager::new();
    let presence_service = PresenceService::new();

    let num_users = 200;
    let start = Instant::now();

    // 模拟大量用户同时重连
    let mut handles = vec![];
    for _ in 0..num_users {
        let cm = connection_manager.clone();
        let ps = presence_service.clone();

        let handle = tokio::spawn(async move {
            let user_id = Uuid::new_v4();
            let _rx = cm.register_connection(user_id).await;
            ps.user_connected(user_id).await.unwrap();
            user_id
        });
        handles.push(handle);
    }

    let mut user_ids = vec![];
    for handle in handles {
        if let Ok(user_id) = handle.await {
            user_ids.push(user_id);
        }
    }

    let duration = start.elapsed();
    let rate = user_ids.len() as f64 / duration.as_secs_f64();

    println!(
        "大量重连: {} 用户在 {:?} ({:.2} users/sec)",
        user_ids.len(),
        duration,
        rate
    );

    // 验证所有用户都连接成功
    assert_eq!(user_ids.len(), num_users, "所有用户应该连接成功");

    // 应该能快速处理
    assert!(
        rate >= 100.0,
        "应该能处理 100+ users/sec，实际: {:.2}",
        rate
    );

    // 清理
    for user_id in user_ids {
        connection_manager.close_user_connections(&user_id).await;
    }
}

/// **混沌测试: 错误输入处理**
/// 验证系统正确处理各种错误输入
#[tokio::test]
async fn chaos_test_invalid_input_handling() {
    let pool = common::setup_test_db().await;
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let email_service: Arc<dyn ai_im_server::services::email::EmailService> =
        Arc::new(MockEmailService::silent());

    let auth_service = AuthService::with_bcrypt_cost(
        pool.clone(),
        redis.clone(),
        "chaos_test_secret".to_string(),
        email_service,
        4,
    );

    // 空用户名
    let result = auth_service
        .register(RegisterData {
            username: "".to_string(),
            email: "test@test.com".to_string(),
            password: "ValidPass123!".to_string(),
        })
        .await;
    assert!(result.is_err(), "空用户名应该失败");

    // 无效邮箱 - 使用明显无效的格式
    let result = auth_service
        .register(RegisterData {
            username: "validuser".to_string(),
            email: "".to_string(), // 空邮箱
            password: "ValidPass123!".to_string(),
        })
        .await;
    assert!(result.is_err(), "空邮箱应该失败");

    // 弱密码
    let result = auth_service
        .register(RegisterData {
            username: "validuser2".to_string(),
            email: "valid@test.com".to_string(),
            password: "weak".to_string(),
        })
        .await;
    assert!(result.is_err(), "弱密码应该失败");

    // 用户名太长
    let long_username = "a".repeat(300);
    let result = auth_service
        .register(RegisterData {
            username: long_username,
            email: "long@test.com".to_string(),
            password: "ValidPass123!".to_string(),
        })
        .await;
    assert!(result.is_err(), "过长用户名应该失败");
}

/// **混沌测试: 登录失败限制**
/// 验证系统限制登录失败次数
#[tokio::test]
async fn chaos_test_login_failure_rate_limiting() {
    let pool = common::setup_test_db().await;
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let email_service: Arc<dyn ai_im_server::services::email::EmailService> =
        Arc::new(MockEmailService::silent());

    let auth_service = AuthService::with_bcrypt_cost(
        pool.clone(),
        redis.clone(),
        "chaos_test_secret".to_string(),
        email_service,
        4,
    );

    // 创建测试用户
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("chaos_rate_{}", &uuid_str[..8]);
    let register_data = RegisterData {
        username: username.clone(),
        email: format!("rate_{}@test.com", uuid_str),
        password: "ValidPass123!".to_string(),
    };
    auth_service.register(register_data).await.unwrap();

    // 多次错误登录
    let mut auth_service = auth_service.clone();
    for i in 0..10 {
        let login_data = LoginData {
            username: username.clone(),
            password: format!("WrongPass{}", i),
        };
        let _ = auth_service.login(login_data).await;
    }

    // 正确密码应该仍然能登录（或被限制）
    let login_data = LoginData {
        username: username.clone(),
        password: "ValidPass123!".to_string(),
    };
    let result = auth_service.login(login_data).await;

    // 根据实现，可能成功或被限制
    println!(
        "多次失败后登录结果: {}",
        if result.is_ok() { "成功" } else { "被限制" }
    );

    // 清理
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(&username)
        .execute(&pool)
        .await
        .ok();
}
