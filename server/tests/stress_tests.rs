//! 压力测试 - 验证系统在极端条件下的行为
//!
//! 测试场景:
//! - 内存压力
//! - CPU 密集操作
//! - 大数据量处理
//! - 长时间运行稳定性

mod common;

use ai_im_server::api::ConnectionManager;
use ai_im_server::models::user::CreateUser;
use ai_im_server::services::auth::{TokenService, UserService};
use ai_im_server::services::presence::PresenceService;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// **压力测试: 大量 Token 生成**
/// 验证 Token 服务在高负载下的稳定性
#[tokio::test]
async fn stress_test_token_generation() {
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let token_service = TokenService::with_redis("stress_test_secret".to_string(), redis);

    let num_tokens = 5000;
    let start = Instant::now();
    let success_count = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];
    for _ in 0..num_tokens {
        let ts = token_service.clone();
        let counter = success_count.clone();

        let handle = tokio::spawn(async move {
            let user_id = Uuid::new_v4();
            if ts.generate_token_pair(&user_id, "stress_user").is_ok() {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let duration = start.elapsed();
    let successful = success_count.load(Ordering::Relaxed);
    let rate = successful as f64 / duration.as_secs_f64();

    println!(
        "Token 生成压力测试: {} 成功 / {} 总数 ({:.2} tokens/sec) in {:?}",
        successful, num_tokens, rate, duration
    );

    assert!(
        successful >= (num_tokens as u64 * 99 / 100),
        "至少 99% 的 token 生成应该成功"
    );
}

/// **压力测试: 大量用户创建**
/// 验证数据库在批量插入时的性能
#[tokio::test]
async fn stress_test_bulk_user_creation() {
    let pool = common::setup_test_db().await;
    let user_service = UserService::new(pool.clone());

    let num_users = 500;
    let start = Instant::now();
    let success_count = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];
    for i in 0..num_users {
        let us = user_service.clone();
        let counter = success_count.clone();

        let handle = tokio::spawn(async move {
            let uuid_str = Uuid::new_v4().to_string().replace("-", "");
            let create_data = CreateUser {
                username: format!("stress_user_{}_{}", i, &uuid_str[..6]),
                email: format!("stress_{}@test.com", uuid_str),
                password_hash: "hashed_password".to_string(),
            };

            if us.create_user(create_data).await.is_ok() {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let duration = start.elapsed();
    let successful = success_count.load(Ordering::Relaxed);
    let rate = successful as f64 / duration.as_secs_f64();

    println!(
        "批量用户创建: {} 成功 / {} 总数 ({:.2} users/sec) in {:?}",
        successful, num_users, rate, duration
    );

    // 清理
    sqlx::query("DELETE FROM users WHERE username LIKE 'stress_user_%'")
        .execute(&pool)
        .await
        .ok();

    assert!(
        successful >= (num_users as u64 * 95 / 100),
        "至少 95% 的用户创建应该成功"
    );
}

/// **压力测试: 连接管理器容量**
/// 验证连接管理器处理大量连接的能力
#[tokio::test]
async fn stress_test_connection_manager_capacity() {
    let connection_manager = ConnectionManager::new();

    let num_connections = 1000;
    let start = Instant::now();

    // 创建大量连接
    let mut receivers = vec![];
    let mut user_ids = vec![];

    for _ in 0..num_connections {
        let user_id = Uuid::new_v4();
        let rx = connection_manager.register_connection(user_id).await;
        receivers.push(rx);
        user_ids.push(user_id);
    }

    let creation_time = start.elapsed();

    // 验证所有连接
    let mut active_count = 0;
    for user_id in &user_ids {
        active_count += connection_manager.connection_count(user_id).await;
    }

    println!(
        "连接管理器容量测试: {} 连接创建于 {:?}",
        active_count, creation_time
    );

    assert_eq!(active_count, num_connections, "所有连接应该被追踪");

    // 批量关闭
    let close_start = Instant::now();
    for user_id in &user_ids {
        connection_manager.close_user_connections(user_id).await;
    }
    let close_time = close_start.elapsed();

    println!("批量关闭 {} 连接于 {:?}", num_connections, close_time);

    // 验证清理
    let mut remaining = 0;
    for user_id in &user_ids {
        remaining += connection_manager.connection_count(user_id).await;
    }
    assert_eq!(remaining, 0, "所有连接应该被清理");
}

/// **压力测试: 在线状态追踪**
/// 验证 Presence 服务处理大量状态变化
#[tokio::test]
async fn stress_test_presence_tracking() {
    let presence_service = PresenceService::new();

    let num_users = 500;
    let start = Instant::now();

    // 大量用户上线
    let mut user_ids = vec![];
    for _ in 0..num_users {
        let user_id = Uuid::new_v4();
        presence_service.user_connected(user_id).await.unwrap();
        user_ids.push(user_id);
    }

    let online_time = start.elapsed();
    println!("{} 用户上线于 {:?}", num_users, online_time);

    // 批量查询状态
    let query_start = Instant::now();
    for user_id in &user_ids {
        let _ = presence_service.get_presence(user_id).await;
    }
    let query_time = query_start.elapsed();
    println!("{} 状态查询于 {:?}", num_users, query_time);

    // 大量用户下线
    let offline_start = Instant::now();
    for user_id in &user_ids {
        presence_service.user_disconnected(*user_id).await.unwrap();
    }
    let offline_time = offline_start.elapsed();
    println!("{} 用户下线于 {:?}", num_users, offline_time);

    // 性能断言
    let total_time = start.elapsed();
    let ops_per_sec = (num_users * 3) as f64 / total_time.as_secs_f64();
    println!("总操作速率: {:.2} ops/sec", ops_per_sec);

    assert!(
        ops_per_sec >= 500.0,
        "应该能处理 500+ ops/sec，实际: {:.2}",
        ops_per_sec
    );
}

/// **压力测试: 批量用户查询**
/// 验证批量查询的性能
#[tokio::test]
async fn stress_test_batch_user_queries() {
    let pool = common::setup_test_db().await;
    let user_service = UserService::new(pool.clone());

    // 创建测试用户
    let num_users = 100;
    let mut user_ids = vec![];

    for i in 0..num_users {
        let uuid_str = Uuid::new_v4().to_string().replace("-", "");
        let create_data = CreateUser {
            username: format!("batch_query_{}_{}", i, &uuid_str[..6]),
            email: format!("batch_{}@test.com", uuid_str),
            password_hash: "hashed".to_string(),
        };
        if let Ok(user) = user_service.create_user(create_data).await {
            user_ids.push(user.id);
        }
    }

    // 批量查询测试
    let num_queries = 50;
    let start = Instant::now();

    for _ in 0..num_queries {
        let _ = user_service.get_profiles_by_ids(&user_ids).await;
    }

    let duration = start.elapsed();
    let avg_query_time = duration / num_queries;

    println!(
        "批量查询 {} 用户: 平均 {:?}/查询 ({} 次查询于 {:?})",
        user_ids.len(),
        avg_query_time,
        num_queries,
        duration
    );

    // 清理
    sqlx::query("DELETE FROM users WHERE username LIKE 'batch_query_%'")
        .execute(&pool)
        .await
        .ok();

    assert!(
        avg_query_time.as_millis() < 100,
        "批量查询应该 < 100ms，实际: {:?}",
        avg_query_time
    );
}

/// **压力测试: 搜索性能**
/// 验证搜索在大数据量下的性能
#[tokio::test]
async fn stress_test_search_performance() {
    let tx = common::setup_test_transaction().await;
    let mut user_service = UserService::with_transaction(tx);

    // 创建大量用户
    let num_users = 200;
    let prefix = format!("search_stress_{}", &Uuid::new_v4().to_string()[..8]);

    for i in 0..num_users {
        let create_data = CreateUser {
            username: format!("{}_{}", prefix, i),
            email: format!("search_{}@test.com", Uuid::new_v4()),
            password_hash: "hashed".to_string(),
        };
        let _ = user_service.create_user(create_data).await;
    }

    // 搜索测试
    let num_searches = 100;
    let start = Instant::now();

    for _ in 0..num_searches {
        let _ = user_service.search_users(&prefix).await;
    }

    let duration = start.elapsed();
    let avg_search_time = duration / num_searches;
    let searches_per_sec = num_searches as f64 / duration.as_secs_f64();

    println!(
        "搜索压力测试: 平均 {:?}/搜索 ({:.2} searches/sec)",
        avg_search_time, searches_per_sec
    );

    assert!(
        avg_search_time.as_millis() < 200,
        "搜索应该 < 200ms，实际: {:?}",
        avg_search_time
    );
}

/// **压力测试: 持续运行稳定性**
/// 验证系统在持续负载下的稳定性
#[tokio::test]
async fn stress_test_sustained_load() {
    let connection_manager = ConnectionManager::new();
    let presence_service = PresenceService::new();

    let duration_secs = 5;
    let start = Instant::now();
    let ops_count = Arc::new(AtomicU64::new(0));

    // 持续创建和销毁连接
    let mut handles = vec![];
    for _ in 0..10 {
        let cm = connection_manager.clone();
        let ps = presence_service.clone();
        let counter = ops_count.clone();

        let handle = tokio::spawn(async move {
            let deadline = Instant::now() + Duration::from_secs(duration_secs);

            while Instant::now() < deadline {
                let user_id = Uuid::new_v4();

                // 连接
                let _rx = cm.register_connection(user_id).await;
                ps.user_connected(user_id).await.unwrap();
                counter.fetch_add(1, Ordering::Relaxed);

                // 短暂保持
                tokio::time::sleep(Duration::from_millis(10)).await;

                // 断开
                cm.close_user_connections(&user_id).await;
                ps.user_disconnected(user_id).await.unwrap();
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let total_ops = ops_count.load(Ordering::Relaxed);
    let actual_duration = start.elapsed();
    let ops_per_sec = total_ops as f64 / actual_duration.as_secs_f64();

    println!(
        "持续负载测试: {} 操作于 {:?} ({:.2} ops/sec)",
        total_ops, actual_duration, ops_per_sec
    );

    assert!(
        ops_per_sec >= 100.0,
        "应该能维持 100+ ops/sec，实际: {:.2}",
        ops_per_sec
    );
}

/// **压力测试: 内存使用**
/// 验证大量对象创建后的内存行为
#[tokio::test]
async fn stress_test_memory_usage() {
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let token_service = TokenService::with_redis("memory_test_secret".to_string(), redis);

    // 生成大量 token
    let num_tokens = 1000;
    let mut tokens = vec![];

    let start = Instant::now();
    for _ in 0..num_tokens {
        let user_id = Uuid::new_v4();
        if let Ok(pair) = token_service.generate_token_pair(&user_id, "mem_test") {
            tokens.push(pair);
        }
    }
    let generation_time = start.elapsed();

    println!(
        "生成 {} tokens 于 {:?}，内存中保持 {} tokens",
        num_tokens,
        generation_time,
        tokens.len()
    );

    // 验证所有 token
    let verify_start = Instant::now();
    let mut valid_count = 0;
    for pair in &tokens {
        if token_service.verify_token(&pair.access_token).is_ok() {
            valid_count += 1;
        }
    }
    let verify_time = verify_start.elapsed();

    println!(
        "验证 {} tokens 于 {:?}，{} 有效",
        tokens.len(),
        verify_time,
        valid_count
    );

    assert_eq!(valid_count, tokens.len(), "所有 token 应该有效");

    // 清理
    drop(tokens);
}
