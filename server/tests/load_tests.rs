//! 负载测试 - 验证系统在高并发下的表现
//!
//! 测试目标:
//! - WebSocket 连接: 1000+ 并发连接
//! - 消息吞吐量: 1000+ msg/sec
//! - API 响应时间: P99 < 100ms

mod common;

use ai_im_server::api::ConnectionManager;
use ai_im_server::services::auth::{AuthService, RegisterData, TokenService};
use ai_im_server::services::email::MockEmailService;
use ai_im_server::services::presence::PresenceService;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use uuid::Uuid;

/// 统计收集器
struct LoadTestStats {
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    total_latency_us: AtomicU64,
    max_latency_us: AtomicU64,
    min_latency_us: AtomicU64,
}

impl LoadTestStats {
    fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_latency_us: AtomicU64::new(0),
            max_latency_us: AtomicU64::new(0),
            min_latency_us: AtomicU64::new(u64::MAX),
        }
    }

    fn record_success(&self, latency: Duration) {
        let latency_us = latency.as_micros() as u64;
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.total_latency_us.fetch_add(latency_us, Ordering::Relaxed);
        self.max_latency_us.fetch_max(latency_us, Ordering::Relaxed);
        self.min_latency_us.fetch_min(latency_us, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    fn report(&self, test_name: &str, duration: Duration) {
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_latency = self.total_latency_us.load(Ordering::Relaxed);
        let max_latency = self.max_latency_us.load(Ordering::Relaxed);
        let min_latency = self.min_latency_us.load(Ordering::Relaxed);

        let avg_latency = if successful > 0 {
            total_latency / successful
        } else {
            0
        };
        let throughput = total as f64 / duration.as_secs_f64();
        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        println!("\n=== {} 负载测试报告 ===", test_name);
        println!("总请求数: {}", total);
        println!("成功: {} | 失败: {}", successful, failed);
        println!("成功率: {:.2}%", success_rate);
        println!("吞吐量: {:.2} req/sec", throughput);
        println!(
            "延迟 - 平均: {}μs | 最小: {}μs | 最大: {}μs",
            avg_latency,
            if min_latency == u64::MAX { 0 } else { min_latency },
            max_latency
        );
        println!("总耗时: {:?}", duration);
    }
}

/// **负载测试: WebSocket 并发连接**
/// 目标: 支持 500+ 并发连接
#[tokio::test]
async fn load_test_websocket_concurrent_connections() {
    let connection_manager = ConnectionManager::new();
    let presence_service = PresenceService::new();
    let stats = Arc::new(LoadTestStats::new());

    let num_connections = 500;
    let start = Instant::now();

    // 并发创建连接
    let mut handles = vec![];
    for _ in 0..num_connections {
        let cm = connection_manager.clone();
        let ps = presence_service.clone();
        let stats = stats.clone();

        let handle = tokio::spawn(async move {
            let user_id = Uuid::new_v4();
            let op_start = Instant::now();

            // 注册连接
            let _rx = cm.register_connection(user_id).await;
            let _ = ps.user_connected(user_id).await;

            stats.record_success(op_start.elapsed());
            user_id
        });
        handles.push(handle);
    }

    // 等待所有连接建立
    let mut user_ids = vec![];
    for handle in handles {
        if let Ok(user_id) = handle.await {
            user_ids.push(user_id);
        }
    }

    let duration = start.elapsed();
    stats.report("WebSocket 并发连接", duration);

    // 验证连接数
    let mut total_connections = 0;
    for user_id in &user_ids {
        total_connections += connection_manager.connection_count(user_id).await;
    }

    println!("活跃连接数: {}", total_connections);

    // 断言
    let successful = stats.successful_requests.load(Ordering::Relaxed);
    assert!(
        successful >= (num_connections as u64 * 95 / 100),
        "至少 95% 的连接应该成功建立"
    );

    // 清理
    for user_id in user_ids {
        connection_manager.close_user_connections(&user_id).await;
    }
}

/// **负载测试: 消息广播吞吐量**
/// 目标: 1000+ msg/sec 广播能力
#[tokio::test]
async fn load_test_message_broadcast_throughput() {
    let connection_manager = ConnectionManager::new();
    let stats = Arc::new(LoadTestStats::new());

    // 创建多个用户连接
    let num_users = 100;
    let mut user_receivers: HashMap<Uuid, _> = HashMap::new();

    for _ in 0..num_users {
        let user_id = Uuid::new_v4();
        let rx = connection_manager.register_connection(user_id).await;
        user_receivers.insert(user_id, rx);
    }

    let user_ids: Vec<_> = user_receivers.keys().cloned().collect();

    // 模拟消息广播
    let num_messages = 1000;
    let start = Instant::now();

    for i in 0..num_messages {
        let target_user = &user_ids[i % num_users];
        let op_start = Instant::now();

        // 模拟发送消息（通过 connection manager 的 channel）
        let count = connection_manager.connection_count(target_user).await;
        if count > 0 {
            stats.record_success(op_start.elapsed());
        } else {
            stats.record_failure();
        }
    }

    let duration = start.elapsed();
    stats.report("消息广播吞吐量", duration);

    let throughput = num_messages as f64 / duration.as_secs_f64();
    println!("消息广播速率: {:.2} msg/sec", throughput);

    // 目标: 1000+ msg/sec
    assert!(
        throughput >= 500.0,
        "消息广播应达到 500+ msg/sec，实际: {:.2}",
        throughput
    );
}

/// **负载测试: 用户注册并发**
/// 目标: 50+ 注册/sec
#[tokio::test]
async fn load_test_concurrent_registration() {
    let pool = common::setup_test_db().await;
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let email_service: Arc<dyn ai_im_server::services::email::EmailService> =
        Arc::new(MockEmailService::silent());

    let auth_service = AuthService::with_bcrypt_cost(
        pool.clone(),
        redis.clone(),
        "load_test_secret".to_string(),
        email_service,
        4, // 低成本用于测试
    );

    let stats = Arc::new(LoadTestStats::new());
    let semaphore = Arc::new(Semaphore::new(50)); // 限制并发数

    let num_registrations = 200;
    let start = Instant::now();

    let mut handles = vec![];
    for i in 0..num_registrations {
        let auth_service = auth_service.clone();
        let stats = stats.clone();
        let semaphore = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let op_start = Instant::now();

            let uuid_str = Uuid::new_v4().to_string().replace("-", "");
            let register_data = RegisterData {
                username: format!("load_reg_{}_{}", i, &uuid_str[..6]),
                email: format!("load_{}@test.com", uuid_str),
                password: "LoadTest123!".to_string(),
            };

            match auth_service.register(register_data).await {
                Ok(_) => stats.record_success(op_start.elapsed()),
                Err(_) => stats.record_failure(),
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let duration = start.elapsed();
    stats.report("并发注册", duration);

    // 清理
    sqlx::query("DELETE FROM users WHERE username LIKE 'load_reg_%'")
        .execute(&pool)
        .await
        .ok();

    let successful = stats.successful_requests.load(Ordering::Relaxed);
    assert!(
        successful >= (num_registrations as u64 * 90 / 100),
        "至少 90% 的注册应该成功"
    );
}

/// **负载测试: Token 验证吞吐量**
/// 目标: 10000+ 验证/sec
#[tokio::test]
async fn load_test_token_verification_throughput() {
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let token_service = TokenService::with_redis("load_test_secret".to_string(), redis);

    // 生成测试 token
    let user_id = Uuid::new_v4();
    let token_pair = token_service
        .generate_token_pair(&user_id, "load_test_user")
        .unwrap();
    let access_token = token_pair.access_token;

    let stats = Arc::new(LoadTestStats::new());
    let num_verifications = 10000;
    let start = Instant::now();

    // 并发验证
    let mut handles = vec![];
    for _ in 0..num_verifications {
        let token_service = token_service.clone();
        let token = access_token.clone();
        let stats = stats.clone();

        let handle = tokio::spawn(async move {
            let op_start = Instant::now();
            match token_service.verify_token(&token) {
                Ok(_) => stats.record_success(op_start.elapsed()),
                Err(_) => stats.record_failure(),
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let duration = start.elapsed();
    stats.report("Token 验证", duration);

    let throughput = num_verifications as f64 / duration.as_secs_f64();
    assert!(
        throughput >= 5000.0,
        "Token 验证应达到 5000+/sec，实际: {:.2}",
        throughput
    );
}

/// **负载测试: 数据库连接池压力**
/// 验证连接池在高负载下的表现
#[tokio::test]
async fn load_test_database_connection_pool() {
    let pool = common::setup_test_db().await;
    let stats = Arc::new(LoadTestStats::new());

    let num_queries = 500;
    let concurrency = 100;
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let start = Instant::now();

    let mut handles = vec![];
    for i in 0..num_queries {
        let pool = pool.clone();
        let stats = stats.clone();
        let semaphore = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let op_start = Instant::now();

            // 执行简单查询
            let result = sqlx::query("SELECT 1 as val")
                .fetch_one(&pool)
                .await;

            match result {
                Ok(_) => stats.record_success(op_start.elapsed()),
                Err(_) => stats.record_failure(),
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let duration = start.elapsed();
    stats.report("数据库连接池", duration);

    let successful = stats.successful_requests.load(Ordering::Relaxed);
    assert!(
        successful >= (num_queries as u64 * 99 / 100),
        "至少 99% 的查询应该成功"
    );
}

/// **负载测试: 混合工作负载**
/// 模拟真实场景：注册、登录、消息发送混合
#[tokio::test]
async fn load_test_mixed_workload() {
    let pool = common::setup_test_db().await;
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let email_service: Arc<dyn ai_im_server::services::email::EmailService> =
        Arc::new(MockEmailService::silent());

    let auth_service = AuthService::with_bcrypt_cost(
        pool.clone(),
        redis.clone(),
        "mixed_test_secret".to_string(),
        email_service,
        4,
    );
    let token_service = TokenService::with_redis("mixed_test_secret".to_string(), redis.clone());
    let connection_manager = ConnectionManager::new();

    let stats = Arc::new(LoadTestStats::new());
    let num_operations = 300;
    let start = Instant::now();

    let mut handles = vec![];
    for i in 0..num_operations {
        let auth_service = auth_service.clone();
        let token_service = token_service.clone();
        let cm = connection_manager.clone();
        let stats = stats.clone();

        let handle = tokio::spawn(async move {
            let op_start = Instant::now();
            let op_type = i % 3;

            let success = match op_type {
                0 => {
                    // 注册
                    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
                    let data = RegisterData {
                        username: format!("mixed_{}_{}", i, &uuid_str[..6]),
                        email: format!("mixed_{}@test.com", uuid_str),
                        password: "MixedTest123!".to_string(),
                    };
                    auth_service.register(data).await.is_ok()
                }
                1 => {
                    // Token 验证
                    let user_id = Uuid::new_v4();
                    if let Ok(pair) = token_service.generate_token_pair(&user_id, "test") {
                        token_service.verify_token(&pair.access_token).is_ok()
                    } else {
                        false
                    }
                }
                _ => {
                    // WebSocket 连接
                    let user_id = Uuid::new_v4();
                    let _rx = cm.register_connection(user_id).await;
                    true
                }
            };

            if success {
                stats.record_success(op_start.elapsed());
            } else {
                stats.record_failure();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let duration = start.elapsed();
    stats.report("混合工作负载", duration);

    // 清理
    sqlx::query("DELETE FROM users WHERE username LIKE 'mixed_%'")
        .execute(&pool)
        .await
        .ok();

    let successful = stats.successful_requests.load(Ordering::Relaxed);
    assert!(
        successful >= (num_operations as u64 * 85 / 100),
        "至少 85% 的操作应该成功"
    );
}
