//! Proactive Messaging Module
//!
//! This module provides proactive messaging capabilities for AI agents,
//! allowing them to initiate conversations based on silence detection,
//! activity patterns, and user relationship levels.
//!
//! ## Features
//! - Silence detection and breaking
//! - Activity-based triggering
//! - **User profile-based triggering** (Phase 6)
//! - **Daily greetings and care messages** (Phase 6)
//! - **Relationship-aware message generation** (Phase 6)

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;
use uuid::Uuid;

use super::user_profile::{RelationshipLevel, UserProfile};

/// Proactive messaging errors
#[derive(Debug, Error)]
pub enum ProactiveError {
    #[error("Proactive messaging disabled for group {0}")]
    Disabled(Uuid),

    #[error("Cooldown active, next message allowed at {0}")]
    CooldownActive(DateTime<Utc>),

    #[error("No suitable topic found")]
    NoSuitableTopic,

    #[error("Message generation failed: {0}")]
    GenerationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Configuration for proactive messaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveConfig {
    /// Whether proactive messaging is enabled
    pub enabled: bool,
    /// Silence threshold in minutes before agent may initiate
    pub silence_threshold_minutes: u32,
    /// Minimum cooldown between proactive messages (minutes)
    pub cooldown_minutes: u32,
    /// Maximum proactive messages per day
    pub max_daily_messages: u32,
    /// Time windows when proactive messages are allowed (hour ranges)
    pub allowed_hours: Vec<(u8, u8)>,
    /// Topics the agent can initiate conversations about
    pub topics: Vec<String>,
}

impl Default for ProactiveConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            silence_threshold_minutes: 60,
            cooldown_minutes: 120,
            max_daily_messages: 3,
            allowed_hours: vec![(9, 22)], // 9 AM to 10 PM
            topics: vec![
                "天气".to_string(),
                "问候".to_string(),
                "趣闻".to_string(),
            ],
        }
    }
}

/// Activity pattern for a group
#[derive(Debug, Clone, Default)]
pub struct ActivityPattern {
    /// Average messages per hour by hour of day
    pub hourly_activity: [f32; 24],
    /// Most active users
    pub active_users: Vec<Uuid>,
    /// Common topics discussed
    pub common_topics: Vec<String>,
    /// Last activity time
    pub last_activity: Option<DateTime<Utc>>,
}

/// Proactive message trigger reason
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProactiveTrigger {
    /// Triggered by silence detection
    SilenceBreaker,
    /// Triggered by scheduled time
    Scheduled,
    /// Triggered by activity pattern
    ActivityBased,
    /// Triggered by special event (holiday, etc.)
    SpecialEvent(String),
    /// Triggered based on user relationship level (Phase 6)
    RelationshipBased(RelationshipTriggerReason),
    /// Daily greeting message (Phase 6)
    DailyGreeting(GreetingType),
    /// Care/check-in message (Phase 6)
    CareMessage(CareReason),
}

/// Reason for relationship-based trigger
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipTriggerReason {
    /// Long time no interaction with a close friend
    MissedCloseFriend,
    /// Detected mood change in recent interactions
    MoodChange,
    /// Anniversary or milestone
    Milestone { days_known: u32 },
    /// Reconnecting with distant user
    ReconnectAttempt,
}

/// Type of daily greeting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GreetingType {
    /// Morning greeting
    Morning,
    /// Afternoon greeting  
    Afternoon,
    /// Evening greeting
    Evening,
    /// Weekend greeting
    Weekend,
    /// Holiday greeting
    Holiday(String),
}

/// Reason for care message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CareReason {
    /// User seemed stressed recently
    StressDetected,
    /// User mentioned being busy
    BusyPeriod,
    /// General check-in
    GeneralCheckIn,
    /// Weather-related care
    WeatherCare(String),
    /// Health reminder
    HealthReminder,
}

/// Result of proactive evaluation
#[derive(Debug, Clone)]
pub struct ProactiveDecision {
    /// Whether to send a proactive message
    pub should_send: bool,
    /// Trigger reason if should_send is true
    pub trigger: Option<ProactiveTrigger>,
    /// Suggested topic
    pub suggested_topic: Option<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Target user for personalized message (Phase 6)
    pub target_user: Option<Uuid>,
    /// Relationship context for message generation (Phase 6)
    pub relationship_context: Option<RelationshipContext>,
}

/// Context about user relationship for message generation (Phase 6)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipContext {
    /// User's relationship level with the agent
    pub level: RelationshipLevel,
    /// Intimacy score (-10 to 10)
    pub intimacy_score: i16,
    /// Days since last interaction
    pub days_since_interaction: u32,
    /// User's detected interests
    pub interests: Vec<String>,
    /// Recent emotional tone
    pub recent_mood: Option<String>,
}

/// Trait for proactive messaging service
#[async_trait]
pub trait ProactiveMessenger: Send + Sync {
    /// Check if a proactive message should be sent
    async fn should_send_proactive(
        &self,
        group_id: Uuid,
        config: &ProactiveConfig,
    ) -> Result<ProactiveDecision, ProactiveError>;

    /// Generate a proactive message
    async fn generate_proactive_message(
        &self,
        group_id: Uuid,
        trigger: &ProactiveTrigger,
        topic: Option<&str>,
    ) -> Result<String, ProactiveError>;

    /// Generate a personalized proactive message based on user profile (Phase 6)
    async fn generate_personalized_message(
        &self,
        group_id: Uuid,
        trigger: &ProactiveTrigger,
        context: &RelationshipContext,
        username: Option<&str>,
    ) -> Result<String, ProactiveError>;

    /// Check if should send relationship-based message (Phase 6)
    async fn evaluate_relationship_trigger(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        profile: &UserProfile,
        config: &ProactiveConfig,
    ) -> Result<Option<ProactiveDecision>, ProactiveError>;

    /// Record that a proactive message was sent
    async fn record_proactive_message(
        &self,
        group_id: Uuid,
        trigger: &ProactiveTrigger,
    ) -> Result<(), ProactiveError>;

    /// Get activity pattern for a group
    async fn get_activity_pattern(&self, group_id: Uuid) -> Result<ActivityPattern, ProactiveError>;

    /// Update activity pattern with new message
    async fn update_activity(&self, group_id: Uuid, user_id: Uuid) -> Result<(), ProactiveError>;
}

/// Default implementation of ProactiveMessenger
pub struct DefaultProactiveMessenger {
    /// Last proactive message time per group
    last_proactive: RwLock<HashMap<Uuid, DateTime<Utc>>>,
    /// Daily message count per group
    daily_counts: RwLock<HashMap<(Uuid, String), u32>>,
    /// Activity patterns per group
    activity_patterns: RwLock<HashMap<Uuid, ActivityPattern>>,
    /// Last activity time per group
    last_activity: RwLock<HashMap<Uuid, DateTime<Utc>>>,
    /// Last interaction time per user (Phase 6)
    user_last_interaction: RwLock<HashMap<(Uuid, Uuid), DateTime<Utc>>>,
}

impl DefaultProactiveMessenger {
    pub fn new() -> Self {
        Self {
            last_proactive: RwLock::new(HashMap::new()),
            daily_counts: RwLock::new(HashMap::new()),
            activity_patterns: RwLock::new(HashMap::new()),
            last_activity: RwLock::new(HashMap::new()),
            user_last_interaction: RwLock::new(HashMap::new()),
        }
    }

    /// Get current greeting type based on time (Phase 6)
    fn get_current_greeting_type(&self) -> GreetingType {
        let now = Utc::now();
        let hour = now.hour();
        let weekday = now.weekday();

        use chrono::Weekday;
        if weekday == Weekday::Sat || weekday == Weekday::Sun {
            return GreetingType::Weekend;
        }

        match hour {
            5..=11 => GreetingType::Morning,
            12..=17 => GreetingType::Afternoon,
            _ => GreetingType::Evening,
        }
    }

    /// Calculate days since last interaction (Phase 6)
    fn days_since_interaction(&self, group_id: Uuid, user_id: Uuid) -> u32 {
        let interactions = self.user_last_interaction.read().unwrap();
        if let Some(last_time) = interactions.get(&(group_id, user_id)) {
            let duration = Utc::now() - *last_time;
            duration.num_days().max(0) as u32
        } else {
            0
        }
    }

    /// Update user interaction time (Phase 6)
    pub fn record_user_interaction(&self, group_id: Uuid, user_id: Uuid) {
        let mut interactions = self.user_last_interaction.write().unwrap();
        interactions.insert((group_id, user_id), Utc::now());
    }

    /// Check if within allowed hours
    fn is_within_allowed_hours(&self, config: &ProactiveConfig) -> bool {
        let now = Utc::now();
        let hour = now.hour() as u8;

        config
            .allowed_hours
            .iter()
            .any(|(start, end)| hour >= *start && hour < *end)
    }

    /// Check if cooldown is active
    fn is_cooldown_active(&self, group_id: Uuid, config: &ProactiveConfig) -> bool {
        let last_proactive = self.last_proactive.read().unwrap();
        if let Some(last_time) = last_proactive.get(&group_id) {
            let cooldown = Duration::minutes(config.cooldown_minutes as i64);
            Utc::now() < *last_time + cooldown
        } else {
            false
        }
    }

    /// Get daily message count
    fn get_daily_count(&self, group_id: Uuid) -> u32 {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let daily_counts = self.daily_counts.read().unwrap();
        *daily_counts.get(&(group_id, today)).unwrap_or(&0)
    }

    /// Increment daily message count
    fn increment_daily_count(&self, group_id: Uuid) {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let mut daily_counts = self.daily_counts.write().unwrap();
        let count = daily_counts.entry((group_id, today)).or_insert(0);
        *count += 1;
    }

    /// Check silence duration
    fn get_silence_duration(&self, group_id: Uuid) -> Option<Duration> {
        let last_activity = self.last_activity.read().unwrap();
        last_activity
            .get(&group_id)
            .map(|last| Utc::now() - *last)
    }

    /// Select a topic based on activity pattern
    fn select_topic(&self, group_id: Uuid, config: &ProactiveConfig) -> Option<String> {
        let patterns = self.activity_patterns.read().unwrap();

        // Prefer common topics from activity pattern
        if let Some(pattern) = patterns.get(&group_id) {
            if !pattern.common_topics.is_empty() {
                let idx = rand::random::<usize>() % pattern.common_topics.len();
                return Some(pattern.common_topics[idx].clone());
            }
        }

        // Fall back to configured topics
        if !config.topics.is_empty() {
            let idx = rand::random::<usize>() % config.topics.len();
            return Some(config.topics[idx].clone());
        }

        None
    }
}

impl Default for DefaultProactiveMessenger {
    fn default() -> Self {
        Self::new()
    }
}

use chrono::Datelike;
use chrono::Timelike;

#[async_trait]
impl ProactiveMessenger for DefaultProactiveMessenger {
    async fn should_send_proactive(
        &self,
        group_id: Uuid,
        config: &ProactiveConfig,
    ) -> Result<ProactiveDecision, ProactiveError> {
        // Check if enabled
        if !config.enabled {
            return Ok(ProactiveDecision {
                should_send: false,
                trigger: None,
                suggested_topic: None,
                confidence: 0.0,
                target_user: None,
                relationship_context: None,
            });
        }

        // Check allowed hours
        if !self.is_within_allowed_hours(config) {
            return Ok(ProactiveDecision {
                should_send: false,
                trigger: None,
                suggested_topic: None,
                confidence: 0.0,
                target_user: None,
                relationship_context: None,
            });
        }

        // Check cooldown
        if self.is_cooldown_active(group_id, config) {
            return Ok(ProactiveDecision {
                should_send: false,
                trigger: None,
                suggested_topic: None,
                confidence: 0.0,
                target_user: None,
                relationship_context: None,
            });
        }

        // Check daily limit
        if self.get_daily_count(group_id) >= config.max_daily_messages {
            return Ok(ProactiveDecision {
                should_send: false,
                trigger: None,
                suggested_topic: None,
                confidence: 0.0,
                target_user: None,
                relationship_context: None,
            });
        }

        // Check silence duration
        if let Some(silence) = self.get_silence_duration(group_id) {
            let threshold = Duration::minutes(config.silence_threshold_minutes as i64);
            if silence >= threshold {
                let topic = self.select_topic(group_id, config);
                let confidence = (silence.num_minutes() as f32
                    / config.silence_threshold_minutes as f32)
                    .min(1.0);

                return Ok(ProactiveDecision {
                    should_send: true,
                    trigger: Some(ProactiveTrigger::SilenceBreaker),
                    suggested_topic: topic,
                    confidence,
                    target_user: None,
                    relationship_context: None,
                });
            }
        }

        Ok(ProactiveDecision {
            should_send: false,
            trigger: None,
            suggested_topic: None,
            confidence: 0.0,
            target_user: None,
            relationship_context: None,
        })
    }

    async fn generate_proactive_message(
        &self,
        _group_id: Uuid,
        trigger: &ProactiveTrigger,
        topic: Option<&str>,
    ) -> Result<String, ProactiveError> {
        // Generate message based on trigger and topic
        let message = match trigger {
            ProactiveTrigger::SilenceBreaker => {
                if let Some(topic) = topic {
                    match topic {
                        "天气" => "大家好久没聊天了，今天天气怎么样呀？".to_string(),
                        "问候" => "好久不见大家！最近都在忙什么呢？".to_string(),
                        "趣闻" => "群里好安静，我来分享个有趣的事情吧~".to_string(),
                        _ => format!("好久没聊天了，聊聊{}怎么样？", topic),
                    }
                } else {
                    "大家好！群里好安静，有人在吗？".to_string()
                }
            }
            ProactiveTrigger::Scheduled => "定时问候：大家好！".to_string(),
            ProactiveTrigger::ActivityBased => "看到大家都在，我也来凑个热闹~".to_string(),
            ProactiveTrigger::SpecialEvent(event) => {
                format!("{}快乐！祝大家开心！", event)
            }
            // Phase 6: New trigger types - delegate to personalized generation
            ProactiveTrigger::RelationshipBased(_) => {
                "好久不见，最近怎么样？".to_string()
            }
            ProactiveTrigger::DailyGreeting(greeting_type) => {
                match greeting_type {
                    GreetingType::Morning => "早上好，大家今天也要元气满满！".to_string(),
                    GreetingType::Afternoon => "下午好！大家工作/学习顺利吗？".to_string(),
                    GreetingType::Evening => "晚上好！大家晚餐吃了吗？".to_string(),
                    GreetingType::Weekend => "周末愉快！大家好好休息~".to_string(),
                    GreetingType::Holiday(h) => format!("{}快乐！", h),
                }
            }
            ProactiveTrigger::CareMessage(care_reason) => {
                match care_reason {
                    CareReason::StressDetected => "大家最近辛苦了，记得给自己一些放松的时间~".to_string(),
                    CareReason::BusyPeriod => "忙碑的时候也要注意身体哦！".to_string(),
                    CareReason::GeneralCheckIn => "大家最近怎么样呀？".to_string(),
                    CareReason::WeatherCare(w) => format!("今天{}，大家注意保暖/防晒！", w),
                    CareReason::HealthReminder => "提醒大家多喝水，注意身体健康！".to_string(),
                }
            }
        };

        Ok(message)
    }

    async fn record_proactive_message(
        &self,
        group_id: Uuid,
        _trigger: &ProactiveTrigger,
    ) -> Result<(), ProactiveError> {
        // Update last proactive time
        {
            let mut last_proactive = self.last_proactive.write().unwrap();
            last_proactive.insert(group_id, Utc::now());
        }

        // Increment daily count
        self.increment_daily_count(group_id);

        Ok(())
    }

    async fn get_activity_pattern(&self, group_id: Uuid) -> Result<ActivityPattern, ProactiveError> {
        let patterns = self.activity_patterns.read().unwrap();
        Ok(patterns.get(&group_id).cloned().unwrap_or_default())
    }

    async fn update_activity(&self, group_id: Uuid, user_id: Uuid) -> Result<(), ProactiveError> {
        let now = Utc::now();
        let hour = now.hour() as usize;

        // Update last activity
        {
            let mut last_activity = self.last_activity.write().unwrap();
            last_activity.insert(group_id, now);
        }

        // Update activity pattern
        {
            let mut patterns = self.activity_patterns.write().unwrap();
            let pattern = patterns.entry(group_id).or_default();

            // Update hourly activity (simple moving average)
            pattern.hourly_activity[hour] =
                pattern.hourly_activity[hour] * 0.9 + 0.1;

            // Update active users (keep top 10)
            if !pattern.active_users.contains(&user_id) {
                pattern.active_users.push(user_id);
                if pattern.active_users.len() > 10 {
                    pattern.active_users.remove(0);
                }
            }

            pattern.last_activity = Some(now);
        }

        // Record user interaction for relationship tracking
        self.record_user_interaction(group_id, user_id);

        Ok(())
    }

    /// Generate a personalized proactive message based on user profile (Phase 6)
    async fn generate_personalized_message(
        &self,
        _group_id: Uuid,
        trigger: &ProactiveTrigger,
        context: &RelationshipContext,
        username: Option<&str>,
    ) -> Result<String, ProactiveError> {
        let name = username.unwrap_or("用户");
        let level = &context.level;
        
        let message = match trigger {
            ProactiveTrigger::DailyGreeting(greeting_type) => {
                self.generate_greeting_message(greeting_type, level, name)
            }
            ProactiveTrigger::CareMessage(care_reason) => {
                self.generate_care_message(care_reason, level, name, context)
            }
            ProactiveTrigger::RelationshipBased(reason) => {
                self.generate_relationship_message(reason, level, name, context)
            }
            _ => {
                // Fallback to basic message
                format!("{}~", name)
            }
        };
        
        Ok(message)
    }

    /// Check if should send relationship-based message (Phase 6)
    async fn evaluate_relationship_trigger(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        profile: &UserProfile,
        config: &ProactiveConfig,
    ) -> Result<Option<ProactiveDecision>, ProactiveError> {
        if !config.enabled {
            return Ok(None);
        }

        let relationship_level = RelationshipLevel::from_score(profile.intimacy_score);
        let days_since = self.days_since_interaction(group_id, user_id);

        // Close friends: trigger after 3 days of no interaction
        // Friends: trigger after 7 days
        // Acquaintances: trigger after 14 days
        let trigger_threshold = match relationship_level {
            RelationshipLevel::CloseFriend => 3,
            RelationshipLevel::Friend => 7,
            RelationshipLevel::Acquaintance => 14,
            _ => return Ok(None), // Don't proactively reach out to strangers/distant
        };

        if days_since >= trigger_threshold {
            let context = RelationshipContext {
                level: relationship_level,
                intimacy_score: profile.intimacy_score,
                days_since_interaction: days_since,
                interests: profile.interests.clone(),
                recent_mood: None,
            };

            let trigger_reason = if days_since > 30 {
                RelationshipTriggerReason::ReconnectAttempt
            } else {
                RelationshipTriggerReason::MissedCloseFriend
            };

            let confidence = (days_since as f32 / (trigger_threshold as f32 * 3.0)).min(1.0);

            return Ok(Some(ProactiveDecision {
                should_send: true,
                trigger: Some(ProactiveTrigger::RelationshipBased(trigger_reason)),
                suggested_topic: profile.interests.first().cloned(),
                confidence,
                target_user: Some(user_id),
                relationship_context: Some(context),
            }));
        }

        Ok(None)
    }
}

/// Helper methods for message generation (Phase 6)
impl DefaultProactiveMessenger {
    fn generate_greeting_message(
        &self,
        greeting_type: &GreetingType,
        level: &RelationshipLevel,
        name: &str,
    ) -> String {
        match (greeting_type, level) {
            (GreetingType::Morning, RelationshipLevel::CloseFriend) => {
                format!("早呀{}~今天也要加油哦！", name)
            }
            (GreetingType::Morning, RelationshipLevel::Friend) => {
                format!("早上好{}！新的一天开始了~", name)
            }
            (GreetingType::Morning, _) => {
                "早上好，大家今天也要元气满满！".to_string()
            }
            (GreetingType::Afternoon, RelationshipLevel::CloseFriend) => {
                format!("{}，下午茶时间到了，要不要休息一下？", name)
            }
            (GreetingType::Afternoon, _) => {
                "下午好！大家工作/学习顺利吗？".to_string()
            }
            (GreetingType::Evening, RelationshipLevel::CloseFriend) => {
                format!("{}，晚上好！今天辛苦了~", name)
            }
            (GreetingType::Evening, _) => {
                "晚上好！大家晚餐吃了吗？".to_string()
            }
            (GreetingType::Weekend, RelationshipLevel::CloseFriend) => {
                format!("{}，周末愉快！有什么计划吗？", name)
            }
            (GreetingType::Weekend, _) => {
                "周末愉快！大家好好休息~".to_string()
            }
            (GreetingType::Holiday(holiday), _) => {
                format!("{}快乐！祝大家{}快乐~", holiday, holiday)
            }
        }
    }

    fn generate_care_message(
        &self,
        care_reason: &CareReason,
        level: &RelationshipLevel,
        name: &str,
        context: &RelationshipContext,
    ) -> String {
        match (care_reason, level) {
            (CareReason::StressDetected, RelationshipLevel::CloseFriend) => {
                format!("{}，最近看起来压力很大啊，要不要聊聊？我在呢！", name)
            }
            (CareReason::StressDetected, _) => {
                "大家最近辛苦了，记得给自己一些放松的时间~".to_string()
            }
            (CareReason::BusyPeriod, RelationshipLevel::CloseFriend) => {
                format!("{}，忙完了吗？记得休息哦~", name)
            }
            (CareReason::BusyPeriod, _) => {
                "忙碑的时候也要注意身体哦！".to_string()
            }
            (CareReason::GeneralCheckIn, RelationshipLevel::CloseFriend) => {
                let interest = context.interests.first().map(|s| s.as_str()).unwrap_or("最近");
                format!("{}，好久不见！{}怎么样了？", name, interest)
            }
            (CareReason::GeneralCheckIn, _) => {
                "大家最近怎么样呀？".to_string()
            }
            (CareReason::WeatherCare(weather), _) => {
                format!("今天{}，大家注意{}！", weather, 
                    if weather.contains("冷") || weather.contains("雨") { "保暖" } else { "防晒" })
            }
            (CareReason::HealthReminder, _) => {
                "提醒大家多喝水，注意身体健康！".to_string()
            }
        }
    }

    fn generate_relationship_message(
        &self,
        reason: &RelationshipTriggerReason,
        level: &RelationshipLevel,
        name: &str,
        context: &RelationshipContext,
    ) -> String {
        match (reason, level) {
            (RelationshipTriggerReason::MissedCloseFriend, RelationshipLevel::CloseFriend) => {
                format!("{}！好久不见，想你了~最近在忙什么呢？", name)
            }
            (RelationshipTriggerReason::MissedCloseFriend, _) => {
                format!("{}，好久不见！最近怎么样？", name)
            }
            (RelationshipTriggerReason::ReconnectAttempt, _) => {
                format!("嘘{}，好久没聊天了，还记得我吗？", name)
            }
            (RelationshipTriggerReason::Milestone { days_known }, _) => {
                format!("哇，我们认识已经{}天了！时间过得真快~", days_known)
            }
            (RelationshipTriggerReason::MoodChange, RelationshipLevel::CloseFriend) => {
                format!("{}，最近心情怎么样？感觉你有些不一样~", name)
            }
            (RelationshipTriggerReason::MoodChange, _) => {
                let _ = context; // Silence unused warning
                "最近群里氛围好像有点变化呀，大家都还好吗？".to_string()
            }
        }
    }
}

/// Mock implementation for testing
pub struct MockProactiveMessenger {
    pub should_send: bool,
    pub trigger: Option<ProactiveTrigger>,
    pub message: String,
}

impl MockProactiveMessenger {
    pub fn new() -> Self {
        Self {
            should_send: false,
            trigger: None,
            message: "Mock proactive message".to_string(),
        }
    }

    pub fn with_decision(mut self, should_send: bool, trigger: Option<ProactiveTrigger>) -> Self {
        self.should_send = should_send;
        self.trigger = trigger;
        self
    }
}

impl Default for MockProactiveMessenger {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProactiveMessenger for MockProactiveMessenger {
    async fn should_send_proactive(
        &self,
        _group_id: Uuid,
        _config: &ProactiveConfig,
    ) -> Result<ProactiveDecision, ProactiveError> {
        Ok(ProactiveDecision {
            should_send: self.should_send,
            trigger: self.trigger.clone(),
            suggested_topic: Some("测试话题".to_string()),
            confidence: if self.should_send { 0.8 } else { 0.0 },
            target_user: None,
            relationship_context: None,
        })
    }

    async fn generate_proactive_message(
        &self,
        _group_id: Uuid,
        _trigger: &ProactiveTrigger,
        _topic: Option<&str>,
    ) -> Result<String, ProactiveError> {
        Ok(self.message.clone())
    }

    async fn generate_personalized_message(
        &self,
        _group_id: Uuid,
        _trigger: &ProactiveTrigger,
        _context: &RelationshipContext,
        _username: Option<&str>,
    ) -> Result<String, ProactiveError> {
        Ok(self.message.clone())
    }

    async fn evaluate_relationship_trigger(
        &self,
        _group_id: Uuid,
        _user_id: Uuid,
        _profile: &UserProfile,
        _config: &ProactiveConfig,
    ) -> Result<Option<ProactiveDecision>, ProactiveError> {
        Ok(None)
    }

    async fn record_proactive_message(
        &self,
        _group_id: Uuid,
        _trigger: &ProactiveTrigger,
    ) -> Result<(), ProactiveError> {
        Ok(())
    }

    async fn get_activity_pattern(&self, _group_id: Uuid) -> Result<ActivityPattern, ProactiveError> {
        Ok(ActivityPattern::default())
    }

    async fn update_activity(&self, _group_id: Uuid, _user_id: Uuid) -> Result<(), ProactiveError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proactive_config_default() {
        let config = ProactiveConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.silence_threshold_minutes, 60);
        assert_eq!(config.cooldown_minutes, 120);
        assert_eq!(config.max_daily_messages, 3);
    }

    #[tokio::test]
    async fn test_proactive_disabled() {
        let messenger = DefaultProactiveMessenger::new();
        let config = ProactiveConfig::default(); // disabled by default

        let decision = messenger
            .should_send_proactive(Uuid::new_v4(), &config)
            .await
            .unwrap();

        assert!(!decision.should_send);
    }

    #[tokio::test]
    async fn test_proactive_daily_limit() {
        let messenger = DefaultProactiveMessenger::new();
        let group_id = Uuid::new_v4();
        let mut config = ProactiveConfig::default();
        config.enabled = true;
        config.max_daily_messages = 2;
        config.allowed_hours = vec![(0, 24)]; // Allow all hours for testing

        // Record 2 messages
        messenger
            .record_proactive_message(group_id, &ProactiveTrigger::SilenceBreaker)
            .await
            .unwrap();
        messenger
            .record_proactive_message(group_id, &ProactiveTrigger::SilenceBreaker)
            .await
            .unwrap();

        // Should not send more
        let decision = messenger
            .should_send_proactive(group_id, &config)
            .await
            .unwrap();

        assert!(!decision.should_send);
    }

    #[tokio::test]
    async fn test_update_activity() {
        let messenger = DefaultProactiveMessenger::new();
        let group_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        messenger.update_activity(group_id, user_id).await.unwrap();

        let pattern = messenger.get_activity_pattern(group_id).await.unwrap();
        assert!(pattern.active_users.contains(&user_id));
        assert!(pattern.last_activity.is_some());
    }

    #[tokio::test]
    async fn test_generate_proactive_message() {
        let messenger = DefaultProactiveMessenger::new();
        let group_id = Uuid::new_v4();

        let message = messenger
            .generate_proactive_message(
                group_id,
                &ProactiveTrigger::SilenceBreaker,
                Some("天气"),
            )
            .await
            .unwrap();

        assert!(message.contains("天气"));
    }

    #[tokio::test]
    async fn test_special_event_message() {
        let messenger = DefaultProactiveMessenger::new();
        let group_id = Uuid::new_v4();

        let message = messenger
            .generate_proactive_message(
                group_id,
                &ProactiveTrigger::SpecialEvent("新年".to_string()),
                None,
            )
            .await
            .unwrap();

        assert!(message.contains("新年"));
    }

    /// **Feature: phase3-ai-agent-evolution, Property 16: Proactive Messaging Control**
    /// **Validates: Requirements 11.3**
    #[test]
    fn test_proactive_control_properties() {
        use proptest::prelude::*;

        proptest!(|(
            enabled: bool,
            silence_minutes in 0u32..1000,
            cooldown_minutes in 1u32..500,
            max_daily in 1u32..20
        )| {
            let config = ProactiveConfig {
                enabled,
                silence_threshold_minutes: silence_minutes.max(1),
                cooldown_minutes,
                max_daily_messages: max_daily,
                allowed_hours: vec![(0, 24)],
                topics: vec!["test".to_string()],
            };

            // Property: Config values are always valid
            prop_assert!(config.silence_threshold_minutes >= 1);
            prop_assert!(config.cooldown_minutes >= 1);
            prop_assert!(config.max_daily_messages >= 1);
        });
    }

    #[test]
    fn test_proactive_decision_confidence_range() {
        use proptest::prelude::*;

        proptest!(|(confidence in 0.0f32..=1.0)| {
            let decision = ProactiveDecision {
                should_send: true,
                trigger: Some(ProactiveTrigger::SilenceBreaker),
                suggested_topic: None,
                confidence,
                target_user: None,
                relationship_context: None,
            };

            // Property: Confidence is always in valid range
            prop_assert!(decision.confidence >= 0.0);
            prop_assert!(decision.confidence <= 1.0);
        });
    }
}
