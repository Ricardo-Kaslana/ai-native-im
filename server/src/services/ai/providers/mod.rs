//! Provider Manager Module
//!
//! This module implements the multi-provider LLM management system with:
//! - Domestic provider priority (国内服务商优先)
//! - Automatic fallback on failure
//! - Provider health tracking
//!
//! **Feature: phase3-ai-agent-evolution**
//! **Requirements: 16.1, 16.2, 16.3**

mod provider_manager;

pub use provider_manager::*;
