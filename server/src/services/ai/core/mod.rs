//! Core Agent Architecture Module
//!
//! This module implements the three-layer cognitive architecture:
//! - Perception Layer: Converts raw inputs to AgentEvent
//! - Cognition Layer: AgentCore trait for decision making
//! - Action Layer: Executes AgentAction outputs
//!
//! The architecture follows the "Brain, Senses, and Limbs" (大脑、感官与手脚) design pattern.

mod agent_core;
mod default_agent;
mod events;

pub use agent_core::*;
pub use default_agent::*;
pub use events::*;
