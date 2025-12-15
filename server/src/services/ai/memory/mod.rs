//! Memory System Module
//!
//! Provides memory storage and retrieval for AI agents.
//!
//! Features:
//! - Vector-based semantic memory storage
//! - Relevance-based retrieval
//! - Conversation context management
//! - Memory importance scoring
//!
//! **Feature: phase3-ai-agent-evolution**
//! **Requirements: 10.1, 10.2**

mod vector_store;

pub use vector_store::*;

// Re-export core types
pub use crate::services::ai::core::{
    ContextMessage, ContextRole, Memory, MemoryError, MemoryMetadata, MemoryStore,
};
