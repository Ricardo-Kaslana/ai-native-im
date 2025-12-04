//! Metrics Module
//! 
//! Tracks message system metrics for monitoring and observability.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Message system metrics
#[derive(Debug, Default)]
pub struct MessageMetrics {
    /// Total messages processed
    pub total_messages: AtomicU64,
    /// Total messages delivered
    pub messages_delivered: AtomicU64,
    /// Total messages failed
    pub messages_failed: AtomicU64,
    /// Total duplicate messages
    pub duplicates_detected: AtomicU64,
    /// Total connections established
    pub connections_established: AtomicU64,
    /// Total connections closed
    pub connections_closed: AtomicU64,
    /// Current active connections
    pub active_connections: AtomicU64,
    /// Latency samples for percentile calculation
    latency_samples: Arc<RwLock<Vec<u64>>>,
}

impl MessageMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self {
            total_messages: AtomicU64::new(0),
            messages_delivered: AtomicU64::new(0),
            messages_failed: AtomicU64::new(0),
            duplicates_detected: AtomicU64::new(0),
            connections_established: AtomicU64::new(0),
            connections_closed: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            latency_samples: Arc::new(RwLock::new(Vec::with_capacity(1000))),
        }
    }

    /// Record a message processed
    pub fn record_message(&self) {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a message delivered
    pub fn record_delivery(&self) {
        self.messages_delivered.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a message failure
    pub fn record_failure(&self) {
        self.messages_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a duplicate detected
    pub fn record_duplicate(&self) {
        self.duplicates_detected.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a connection established
    pub fn record_connection_established(&self) {
        self.connections_established.fetch_add(1, Ordering::Relaxed);
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a connection closed
    pub fn record_connection_closed(&self) {
        self.connections_closed.fetch_add(1, Ordering::Relaxed);
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Record message latency
    pub async fn record_latency(&self, latency_ms: u64) {
        let mut samples = self.latency_samples.write().await;
        samples.push(latency_ms);
        
        // Keep only last 1000 samples
        if samples.len() > 1000 {
            samples.remove(0);
        }
    }

    /// Get current metrics snapshot
    pub async fn snapshot(&self) -> MetricsSnapshot {
        let samples = self.latency_samples.read().await;
        let (avg, p50, p95, p99) = calculate_percentiles(&samples);
        
        MetricsSnapshot {
            total_messages: self.total_messages.load(Ordering::Relaxed),
            messages_delivered: self.messages_delivered.load(Ordering::Relaxed),
            messages_failed: self.messages_failed.load(Ordering::Relaxed),
            duplicates_detected: self.duplicates_detected.load(Ordering::Relaxed),
            connections_established: self.connections_established.load(Ordering::Relaxed),
            connections_closed: self.connections_closed.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            avg_latency_ms: avg,
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
        }
    }

    /// Get error rate
    pub fn error_rate(&self) -> f64 {
        let total = self.total_messages.load(Ordering::Relaxed);
        let failed = self.messages_failed.load(Ordering::Relaxed);
        
        if total == 0 {
            0.0
        } else {
            failed as f64 / total as f64
        }
    }

    /// Get throughput (messages per second) - requires external timing
    pub fn throughput(&self, duration: Duration) -> f64 {
        let total = self.total_messages.load(Ordering::Relaxed);
        let secs = duration.as_secs_f64();
        
        if secs == 0.0 {
            0.0
        } else {
            total as f64 / secs
        }
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        self.total_messages.store(0, Ordering::Relaxed);
        self.messages_delivered.store(0, Ordering::Relaxed);
        self.messages_failed.store(0, Ordering::Relaxed);
        self.duplicates_detected.store(0, Ordering::Relaxed);
        self.connections_established.store(0, Ordering::Relaxed);
        self.connections_closed.store(0, Ordering::Relaxed);
        self.active_connections.store(0, Ordering::Relaxed);
        
        let mut samples = self.latency_samples.write().await;
        samples.clear();
    }
}

/// Metrics snapshot for reporting
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub total_messages: u64,
    pub messages_delivered: u64,
    pub messages_failed: u64,
    pub duplicates_detected: u64,
    pub connections_established: u64,
    pub connections_closed: u64,
    pub active_connections: u64,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
}

impl MetricsSnapshot {
    /// Convert to Prometheus format
    pub fn to_prometheus(&self) -> String {
        format!(
            r#"# HELP message_total Total messages processed
# TYPE message_total counter
message_total {}

# HELP message_delivered Total messages delivered
# TYPE message_delivered counter
message_delivered {}

# HELP message_failed Total messages failed
# TYPE message_failed counter
message_failed {}

# HELP message_duplicates Total duplicate messages detected
# TYPE message_duplicates counter
message_duplicates {}

# HELP connections_established Total connections established
# TYPE connections_established counter
connections_established {}

# HELP connections_closed Total connections closed
# TYPE connections_closed counter
connections_closed {}

# HELP connections_active Current active connections
# TYPE connections_active gauge
connections_active {}

# HELP message_latency_avg Average message latency in milliseconds
# TYPE message_latency_avg gauge
message_latency_avg {}

# HELP message_latency_p50 50th percentile message latency in milliseconds
# TYPE message_latency_p50 gauge
message_latency_p50 {}

# HELP message_latency_p95 95th percentile message latency in milliseconds
# TYPE message_latency_p95 gauge
message_latency_p95 {}

# HELP message_latency_p99 99th percentile message latency in milliseconds
# TYPE message_latency_p99 gauge
message_latency_p99 {}
"#,
            self.total_messages,
            self.messages_delivered,
            self.messages_failed,
            self.duplicates_detected,
            self.connections_established,
            self.connections_closed,
            self.active_connections,
            self.avg_latency_ms,
            self.p50_latency_ms,
            self.p95_latency_ms,
            self.p99_latency_ms,
        )
    }
}

/// Calculate percentiles from samples
fn calculate_percentiles(samples: &[u64]) -> (f64, f64, f64, f64) {
    if samples.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }

    let mut sorted = samples.to_vec();
    sorted.sort_unstable();

    let len = sorted.len();
    let avg = sorted.iter().sum::<u64>() as f64 / len as f64;
    let p50 = sorted[len * 50 / 100] as f64;
    let p95 = sorted[len * 95 / 100] as f64;
    let p99 = sorted[len * 99 / 100] as f64;

    (avg, p50, p95, p99)
}

/// Latency timer for measuring operation duration
pub struct LatencyTimer {
    start: Instant,
    metrics: Arc<MessageMetrics>,
}

impl LatencyTimer {
    /// Start a new latency timer
    pub fn start(metrics: Arc<MessageMetrics>) -> Self {
        Self {
            start: Instant::now(),
            metrics,
        }
    }

    /// Stop the timer and record latency
    pub async fn stop(self) {
        let latency_ms = self.start.elapsed().as_millis() as u64;
        self.metrics.record_latency(latency_ms).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_message() {
        let metrics = MessageMetrics::new();
        
        assert_eq!(metrics.total_messages.load(Ordering::Relaxed), 0);
        
        metrics.record_message();
        metrics.record_message();
        
        assert_eq!(metrics.total_messages.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn test_record_connection() {
        let metrics = MessageMetrics::new();
        
        metrics.record_connection_established();
        metrics.record_connection_established();
        
        assert_eq!(metrics.active_connections.load(Ordering::Relaxed), 2);
        
        metrics.record_connection_closed();
        
        assert_eq!(metrics.active_connections.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_error_rate() {
        let metrics = MessageMetrics::new();
        
        // No messages yet
        assert_eq!(metrics.error_rate(), 0.0);
        
        // 10 messages, 2 failures = 20% error rate
        for _ in 0..10 {
            metrics.record_message();
        }
        metrics.record_failure();
        metrics.record_failure();
        
        assert!((metrics.error_rate() - 0.2).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_latency_recording() {
        let metrics = MessageMetrics::new();
        
        metrics.record_latency(10).await;
        metrics.record_latency(20).await;
        metrics.record_latency(30).await;
        
        let snapshot = metrics.snapshot().await;
        assert!(snapshot.avg_latency_ms > 0.0);
    }

    #[tokio::test]
    async fn test_snapshot() {
        let metrics = MessageMetrics::new();
        
        metrics.record_message();
        metrics.record_delivery();
        metrics.record_connection_established();
        
        let snapshot = metrics.snapshot().await;
        
        assert_eq!(snapshot.total_messages, 1);
        assert_eq!(snapshot.messages_delivered, 1);
        assert_eq!(snapshot.active_connections, 1);
    }

    #[tokio::test]
    async fn test_prometheus_format() {
        let metrics = MessageMetrics::new();
        metrics.record_message();
        
        let snapshot = metrics.snapshot().await;
        let prometheus = snapshot.to_prometheus();
        
        assert!(prometheus.contains("message_total 1"));
        assert!(prometheus.contains("# TYPE message_total counter"));
    }

    #[tokio::test]
    async fn test_reset() {
        let metrics = MessageMetrics::new();
        
        metrics.record_message();
        metrics.record_connection_established();
        
        assert_eq!(metrics.total_messages.load(Ordering::Relaxed), 1);
        
        metrics.reset().await;
        
        assert_eq!(metrics.total_messages.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.active_connections.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_calculate_percentiles_empty() {
        let (avg, p50, p95, p99) = calculate_percentiles(&[]);
        
        assert_eq!(avg, 0.0);
        assert_eq!(p50, 0.0);
        assert_eq!(p95, 0.0);
        assert_eq!(p99, 0.0);
    }

    #[test]
    fn test_calculate_percentiles() {
        let samples: Vec<u64> = (1..=100).collect();
        let (avg, p50, p95, p99) = calculate_percentiles(&samples);
        
        assert!((avg - 50.5).abs() < 0.1);
        // p50 should be around 50-51 depending on calculation method
        assert!(p50 >= 50.0 && p50 <= 51.0);
        assert!(p95 >= 95.0 && p95 <= 96.0);
        assert!(p99 >= 99.0 && p99 <= 100.0);
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property 45: Metrics tracking
        /// All recorded metrics should be accurately tracked.
        #[test]
        fn prop_metrics_tracking(
            message_count in 1u64..100,
            delivery_count in 0u64..50,
            failure_count in 0u64..50,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let metrics = MessageMetrics::new();

                for _ in 0..message_count {
                    metrics.record_message();
                }
                for _ in 0..delivery_count {
                    metrics.record_delivery();
                }
                for _ in 0..failure_count {
                    metrics.record_failure();
                }

                let snapshot = metrics.snapshot().await;
                
                prop_assert_eq!(snapshot.total_messages, message_count);
                prop_assert_eq!(snapshot.messages_delivered, delivery_count);
                prop_assert_eq!(snapshot.messages_failed, failure_count);

                Ok(())
            })?;
        }

        /// Property: Connection tracking accuracy
        #[test]
        fn prop_connection_tracking(
            established in 1u64..50,
            closed in 0u64..50,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let metrics = MessageMetrics::new();
                let closed = closed.min(established); // Can't close more than established

                for _ in 0..established {
                    metrics.record_connection_established();
                }
                for _ in 0..closed {
                    metrics.record_connection_closed();
                }

                let snapshot = metrics.snapshot().await;
                
                prop_assert_eq!(snapshot.connections_established, established);
                prop_assert_eq!(snapshot.connections_closed, closed);
                prop_assert_eq!(snapshot.active_connections, established - closed);

                Ok(())
            })?;
        }

        /// Property: Error rate calculation
        #[test]
        fn prop_error_rate_calculation(
            total in 1u64..1000,
            failed in 0u64..500,
        ) {
            let metrics = MessageMetrics::new();
            let failed = failed.min(total); // Can't fail more than total

            for _ in 0..total {
                metrics.record_message();
            }
            for _ in 0..failed {
                metrics.record_failure();
            }

            let expected_rate = failed as f64 / total as f64;
            let actual_rate = metrics.error_rate();

            prop_assert!((actual_rate - expected_rate).abs() < 0.0001);
        }

        /// Property: Latency samples are bounded
        #[test]
        fn prop_latency_samples_bounded(
            sample_count in 1usize..2000,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let metrics = MessageMetrics::new();

                for i in 0..sample_count {
                    metrics.record_latency(i as u64).await;
                }

                let samples = metrics.latency_samples.read().await;
                // Should never exceed 1000 samples
                prop_assert!(samples.len() <= 1000);

                Ok(())
            })?;
        }

        /// Property 47: Prometheus metrics exposure
        /// Prometheus output should contain all required metrics.
        #[test]
        fn prop_prometheus_format(
            messages in 0u64..100,
            delivered in 0u64..100,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let metrics = MessageMetrics::new();

                for _ in 0..messages {
                    metrics.record_message();
                }
                for _ in 0..delivered {
                    metrics.record_delivery();
                }

                let snapshot = metrics.snapshot().await;
                let prometheus = snapshot.to_prometheus();

                // Check required metrics are present
                prop_assert!(prometheus.contains("message_total"));
                prop_assert!(prometheus.contains("message_delivered"));
                prop_assert!(prometheus.contains("message_failed"));
                prop_assert!(prometheus.contains("connections_active"));
                prop_assert!(prometheus.contains("message_latency_avg"));
                prop_assert!(prometheus.contains("message_latency_p50"));
                prop_assert!(prometheus.contains("message_latency_p95"));
                prop_assert!(prometheus.contains("message_latency_p99"));

                // Check TYPE declarations
                prop_assert!(prometheus.contains("# TYPE message_total counter"));
                prop_assert!(prometheus.contains("# TYPE connections_active gauge"));

                Ok(())
            })?;
        }

        /// Property: Reset clears all metrics
        #[test]
        fn prop_reset_clears_all(
            messages in 1u64..100,
            connections in 1u64..50,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let metrics = MessageMetrics::new();

                for _ in 0..messages {
                    metrics.record_message();
                }
                for _ in 0..connections {
                    metrics.record_connection_established();
                }

                // Verify non-zero before reset
                prop_assert!(metrics.total_messages.load(Ordering::Relaxed) > 0);

                metrics.reset().await;

                let snapshot = metrics.snapshot().await;
                prop_assert_eq!(snapshot.total_messages, 0);
                prop_assert_eq!(snapshot.messages_delivered, 0);
                prop_assert_eq!(snapshot.messages_failed, 0);
                prop_assert_eq!(snapshot.active_connections, 0);

                Ok(())
            })?;
        }
    }
}
