use sqlx::{PgPool, Postgres, Transaction};
use std::env;

/// Setup test database connection pool
pub async fn setup_test_db() -> PgPool {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://dev_user:dev_password@localhost:5432/ai_im_dev".to_string()
    });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Setup a test transaction that will be rolled back after the test
/// This provides complete test isolation without manual cleanup
pub async fn setup_test_transaction() -> Transaction<'static, Postgres> {
    let pool = setup_test_db().await;
    pool.begin()
        .await
        .expect("Failed to begin transaction")
}

/// Deprecated: Use setup_test_transaction() instead for better test isolation
#[deprecated(note = "Use setup_test_transaction() for better test isolation")]
pub async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM users WHERE username LIKE 'test_%'")
        .execute(pool)
        .await
        .ok();
}
