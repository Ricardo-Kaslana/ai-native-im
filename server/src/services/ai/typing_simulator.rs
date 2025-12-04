use async_trait::async_trait;
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

/// Maximum typing delay in milliseconds
const MAX_TYPING_DELAY_MS: u64 = 10_000;

/// Milliseconds per character for typing simulation
const MS_PER_CHAR: u64 = 50;

/// Typing simulator trait
#[async_trait]
pub trait TypingSimulator: Send + Sync {
    /// Calculate typing delay for a message
    fn calculate_delay(&self, message: &str) -> Duration;

    /// Simulate typing with delay
    async fn simulate_typing(&self, message: &str);
}

/// Default typing simulator implementation
pub struct DefaultTypingSimulator;

impl DefaultTypingSimulator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultTypingSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TypingSimulator for DefaultTypingSimulator {
    fn calculate_delay(&self, message: &str) -> Duration {
        let char_count = message.chars().count() as u64;
        let delay_ms = (char_count * MS_PER_CHAR).min(MAX_TYPING_DELAY_MS);
        Duration::from_millis(delay_ms)
    }

    async fn simulate_typing(&self, message: &str) {
        let delay = self.calculate_delay(message);
        debug!(
            "Simulating typing for {} chars, delay: {:?}",
            message.chars().count(),
            delay
        );
        sleep(delay).await;
    }
}

/// Fast typing simulator for testing (no actual delay)
pub struct FastTypingSimulator;

impl FastTypingSimulator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FastTypingSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TypingSimulator for FastTypingSimulator {
    fn calculate_delay(&self, message: &str) -> Duration {
        let char_count = message.chars().count() as u64;
        let delay_ms = (char_count * MS_PER_CHAR).min(MAX_TYPING_DELAY_MS);
        Duration::from_millis(delay_ms)
    }

    async fn simulate_typing(&self, _message: &str) {
        // No delay for testing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_delay_short_message() {
        let simulator = DefaultTypingSimulator::new();
        let delay = simulator.calculate_delay("Hello");
        assert_eq!(delay, Duration::from_millis(5 * 50)); // 5 chars * 50ms
    }

    #[test]
    fn test_calculate_delay_medium_message() {
        let simulator = DefaultTypingSimulator::new();
        let message = "This is a medium length message for testing.";
        let delay = simulator.calculate_delay(message);
        let expected = Duration::from_millis(message.chars().count() as u64 * 50);
        assert_eq!(delay, expected);
    }

    #[test]
    fn test_calculate_delay_capped_at_max() {
        let simulator = DefaultTypingSimulator::new();
        // Create a very long message (300 chars = 15000ms, should be capped at 10000ms)
        let message = "a".repeat(300);
        let delay = simulator.calculate_delay(&message);
        assert_eq!(delay, Duration::from_millis(MAX_TYPING_DELAY_MS));
    }

    #[test]
    fn test_calculate_delay_empty_message() {
        let simulator = DefaultTypingSimulator::new();
        let delay = simulator.calculate_delay("");
        assert_eq!(delay, Duration::from_millis(0));
    }

    #[test]
    fn test_calculate_delay_unicode() {
        let simulator = DefaultTypingSimulator::new();
        // Unicode characters should be counted correctly
        let message = "你好世界"; // 4 characters
        let delay = simulator.calculate_delay(message);
        assert_eq!(delay, Duration::from_millis(4 * 50));
    }

    #[test]
    fn test_calculate_delay_emoji() {
        let simulator = DefaultTypingSimulator::new();
        let message = "Hello 👋🌍"; // 8 characters (including emojis)
        let delay = simulator.calculate_delay(message);
        // Note: emoji counting depends on how chars() handles them
        assert!(delay.as_millis() > 0);
    }

    #[tokio::test]
    async fn test_fast_simulator_no_delay() {
        let simulator = FastTypingSimulator::new();
        let start = std::time::Instant::now();
        simulator.simulate_typing("This is a test message").await;
        let elapsed = start.elapsed();
        // Should complete almost instantly
        assert!(elapsed < Duration::from_millis(100));
    }

    #[test]
    fn test_delay_boundary_at_200_chars() {
        let simulator = DefaultTypingSimulator::new();
        // 200 chars * 50ms = 10000ms = exactly at cap
        let message = "a".repeat(200);
        let delay = simulator.calculate_delay(&message);
        assert_eq!(delay, Duration::from_millis(10_000));
    }

    #[test]
    fn test_delay_just_under_cap() {
        let simulator = DefaultTypingSimulator::new();
        // 199 chars * 50ms = 9950ms < cap
        let message = "a".repeat(199);
        let delay = simulator.calculate_delay(&message);
        assert_eq!(delay, Duration::from_millis(9950));
    }
}
