use ai_im_server::{
    api, middleware,
    services::auth::*,
    services::email::*,
    services::message::{
        connection_manager::InMemoryConnectionManager as MessageConnectionManager,
        dedup_cache::InMemoryDedupCache,
        message_persister::{MessagePersister, PostgresMessagePersister},
        message_router::MessageRouterImpl,
        websocket_handler::WebSocketHandler,
    },
    services::presence::*,
};
use axum::{middleware as axum_middleware, routing::get, Router};
use redis::Client as RedisClient;
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ai_im_server=debug,tower_http=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting AI Native IM Server...");

    // 加载环境变量
    dotenv::dotenv().ok();

    // 获取配置
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/ai_im".to_string());
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());
    let server_port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    tracing::info!("📊 Connecting to database...");
    // 创建数据库连接池
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("✅ Database connected");

    tracing::info!("📊 Connecting to Redis...");
    // 创建Redis连接
    let redis_client = RedisClient::open(redis_url)?;
    let redis_conn = redis_client.get_connection_manager().await?;

    tracing::info!("✅ Redis connected");

    // 创建服务
    tracing::info!("🔧 Initializing services...");
    
    // 创建连接管理器
    let connection_manager = Arc::new(api::ConnectionManager::new());
    
    // 创建邮件服务（Mock版本用于开发）
    let email_service: Arc<dyn EmailService> = Arc::new(MockEmailService::new());
    
    let auth_service = AuthService::new(
        pool.clone(),
        redis_conn.clone(),
        jwt_secret.clone(),
        email_service,
    );

    // 创建应用状态
    let app_state = api::auth::AppState {
        auth_service: Arc::new(auth_service),
        pool: pool.clone(),
        connection_manager: Some(connection_manager.clone()),
    };

    // 创建JWT中间件（使用带Redis的TokenService以支持黑名单）
    let token_service = TokenService::with_redis(jwt_secret.clone(), redis_conn.clone());
    let jwt_auth = middleware::JwtAuth::new(token_service);

    // 创建在线状态服务
    let presence_service = Arc::new(PresenceService::with_redis(redis_conn.clone()));
    let (presence_tx, _) = broadcast::channel::<PresenceUpdate>(100);
    
    let ws_state = api::WsState {
        presence_service: presence_service.clone(),
        presence_tx: presence_tx.clone(),
        connection_manager: connection_manager.clone(),
    };

    // 创建消息系统组件
    tracing::info!("🔧 Initializing message system...");
    let message_connection_manager = Arc::new(MessageConnectionManager::new());
    let message_persister: Arc<dyn MessagePersister> =
        Arc::new(PostgresMessagePersister::new(pool.clone()));
    let dedup_cache = Arc::new(InMemoryDedupCache::new(Duration::from_secs(300)));
    
    let base_router = Arc::new(
        MessageRouterImpl::new(
            message_connection_manager.clone(),
            dedup_cache,
            message_persister,
        ).with_pool(pool.clone())
    );

    // 初始化AI Agent系统
    tracing::info!("🤖 Initializing AI Agent system...");
    let ai_config = ai_im_server::services::AIAgentSystemConfig::default();
    let mut ai_system = ai_im_server::services::AIAgentSystem::new(pool.clone(), ai_config)
        .await
        .expect("Failed to initialize AI Agent system");
    
    // 使用 AI 增强的消息路由器
    use ai_im_server::services::AIMessageRouter;
    let message_router = Arc::new(
        AIMessageRouter::new(base_router.clone())
            .with_agent_handler(ai_system.message_handler.clone())
            .with_response_channel(ai_system.response_sender())
    );
    
    // 启动AI响应工作器
    let _ai_worker_handle = ai_system.start_response_worker(base_router.clone());
    tracing::info!("✅ AI Agent system initialized");
    
    let token_service_for_ws = Arc::new(TokenService::with_redis(
        jwt_secret.clone(),
        redis_conn.clone(),
    ));
    
    let ws_message_handler = Arc::new(WebSocketHandler::new(
        message_connection_manager,
        message_router,
        token_service_for_ws,
    ));

    tracing::info!("🔧 Setting up routes...");

    // 配置CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 公开路由（不需要认证）
    let public_routes = Router::new()
        .nest("/auth", api::auth_routes(app_state.clone()))
        .route("/", get(root))
        .route("/health", get(health_check));

    // 消息系统路由（WebSocket，使用 token 参数认证）
    let message_ws_router = api::create_ws_router(ws_message_handler);

    // 受保护的路由（需要JWT认证）
    let protected_routes = Router::new()
        .nest("/auth", api::auth_protected_routes(app_state.clone()))
        .nest("/users", api::user_routes(app_state.clone()))
        .nest("/admin", api::admin_routes(app_state.clone()))
        .nest("/groups", api::group_routes(pool.clone()))
        .merge(api::conversation_routes(pool.clone()))
        .merge(api::contact_routes(pool.clone()))
        .merge(api::ai_agent_routes(pool.clone()))
        .layer(axum_middleware::from_fn_with_state(
            jwt_auth.clone(),
            middleware::JwtAuth::verify,
        ));

    // 组合所有路由
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(message_ws_router)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], server_port));
    tracing::info!("🚀 Server listening on http://{}", addr);
    tracing::info!("📝 API Documentation:");
    tracing::info!("   - POST   /auth/register");
    tracing::info!("   - POST   /auth/login");
    tracing::info!("   - POST   /auth/logout");
    tracing::info!("   - POST   /auth/refresh");
    tracing::info!("   - POST   /auth/password/change");
    tracing::info!("   - POST   /auth/password/reset/request");
    tracing::info!("   - POST   /auth/password/reset/confirm");
    tracing::info!("   - GET    /users/me");
    tracing::info!("   - PUT    /users/me");
    tracing::info!("   - GET    /users/:id");
    tracing::info!("   - POST   /users/batch");
    tracing::info!("   - GET    /users/search");
    tracing::info!("   - GET    /admin/stats");
    tracing::info!("   - GET    /admin/users");
    tracing::info!("   - GET    /admin/users/search");
    tracing::info!("   - POST   /admin/users/:id/suspend");
    tracing::info!("   - POST   /admin/users/:id/activate");
    tracing::info!("   - DELETE /admin/users/:id");
    tracing::info!("   - PUT    /admin/users/:id/role");
    tracing::info!("   - WS     /ws (presence)");
    tracing::info!("   - WS     /ws?token=<jwt> (messages)");
    tracing::info!("   - POST   /groups/:group_id/agent (create AI agent)");
    tracing::info!("   - GET    /groups/:group_id/agent (get group agent)");
    tracing::info!("   - GET    /agents/:agent_id (get agent)");
    tracing::info!("   - DELETE /agents/:agent_id (delete agent)");
    tracing::info!("   - PUT    /agents/:agent_id/personality (update personality)");
    tracing::info!("   - GET    /agents/:agent_id/stats (get statistics)");
    tracing::info!("   - POST   /agents/:agent_id/affinity (set user affinity)");
    tracing::info!("✨ Server ready!");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "🚀 AI Native IM Server - Running"
}

async fn health_check() -> &'static str {
    "OK"
}
