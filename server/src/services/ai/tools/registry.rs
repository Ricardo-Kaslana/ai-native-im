//! Tool Registry
//!
//! Manages available tools and provides lookup functionality.

use std::collections::HashMap;
use std::sync::Arc;

use super::{FunctionDefinition, Tool, ToolCall, ToolError, ToolResult};
use tracing::{debug, info, instrument, warn};

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Create a registry with default tools
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(super::CalculatorTool::new()));
        registry.register(Arc::new(super::CurrentTimeTool::new()));
        registry.register(Arc::new(super::WeatherTool::mock()));
        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        info!(tool_name = %name, "Registering tool");
        self.tools.insert(name, tool);
    }

    /// Unregister a tool
    pub fn unregister(&mut self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.remove(name)
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Check if a tool exists
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get all tool names
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Get all available tools (those that are ready to use)
    pub fn available_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools
            .values()
            .filter(|t| t.is_available())
            .cloned()
            .collect()
    }

    /// Generate function definitions for LLM
    pub fn to_function_definitions(&self) -> Vec<FunctionDefinition> {
        self.tools
            .values()
            .filter(|t| t.is_available())
            .map(|tool| FunctionDefinition {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameters(),
            })
            .collect()
    }

    /// Execute a tool call
    #[instrument(skip(self), fields(tool_name = %call.name))]
    pub async fn execute(&self, call: &ToolCall) -> ToolResult {
        debug!(tool_call_id = %call.id, "Executing tool call");

        match self.get(&call.name) {
            Some(tool) => {
                if !tool.is_available() {
                    warn!(tool_name = %call.name, "Tool not available");
                    return ToolResult::failure(
                        &call.id,
                        &call.name,
                        format!("工具 '{}' 当前不可用", call.name),
                    );
                }

                match tool.execute(call.arguments.clone()).await {
                    Ok(result) => {
                        info!(tool_name = %call.name, "Tool executed successfully");
                        ToolResult::success(&call.id, &call.name, result)
                    }
                    Err(e) => {
                        warn!(tool_name = %call.name, error = %e, "Tool execution failed");
                        ToolResult::failure(&call.id, &call.name, e.to_string())
                    }
                }
            }
            None => {
                warn!(tool_name = %call.name, "Tool not found");
                ToolResult::failure(
                    &call.id,
                    &call.name,
                    format!("未找到工具: {}", call.name),
                )
            }
        }
    }

    /// Execute multiple tool calls
    pub async fn execute_all(&self, calls: &[ToolCall]) -> Vec<ToolResult> {
        let mut results = Vec::with_capacity(calls.len());
        for call in calls {
            results.push(self.execute(call).await);
        }
        results
    }

    /// Execute multiple tool calls concurrently
    pub async fn execute_all_concurrent(&self, calls: &[ToolCall]) -> Vec<ToolResult> {
        use futures::future::join_all;

        let futures: Vec<_> = calls.iter().map(|call| self.execute(call)).collect();
        join_all(futures).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock tool for testing
pub struct MockTool {
    name: String,
    description: String,
    response: String,
    should_fail: bool,
}

impl MockTool {
    pub fn new(name: impl Into<String>, response: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: "Mock tool for testing".to_string(),
            response: response.into(),
            should_fail: false,
        }
    }

    pub fn failing(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: "Mock tool that fails".to_string(),
            response: String::new(),
            should_fail: true,
        }
    }
}

#[async_trait::async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<String, ToolError> {
        if self.should_fail {
            Err(ToolError::ExecutionFailed("Mock failure".to_string()))
        } else {
            Ok(self.response.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = ToolRegistry::new();
        assert!(registry.tool_names().is_empty());
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = ToolRegistry::with_defaults();
        assert!(registry.contains("calculator"));
        assert!(registry.contains("current_time"));
        assert!(registry.contains("get_weather"));
    }

    #[test]
    fn test_registry_register_unregister() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool::new("test_tool", "test response"));

        registry.register(tool);
        assert!(registry.contains("test_tool"));

        let removed = registry.unregister("test_tool");
        assert!(removed.is_some());
        assert!(!registry.contains("test_tool"));
    }

    #[tokio::test]
    async fn test_registry_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::new("test", "success")));

        let call = ToolCall {
            id: "call_1".to_string(),
            name: "test".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = registry.execute(&call).await;
        assert!(result.success);
        assert_eq!(result.content, "success");
    }

    #[tokio::test]
    async fn test_registry_execute_not_found() {
        let registry = ToolRegistry::new();

        let call = ToolCall {
            id: "call_1".to_string(),
            name: "nonexistent".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = registry.execute(&call).await;
        assert!(!result.success);
        assert!(result.content.contains("未找到"));
    }

    #[tokio::test]
    async fn test_registry_execute_failure() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::failing("failing_tool")));

        let call = ToolCall {
            id: "call_1".to_string(),
            name: "failing_tool".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = registry.execute(&call).await;
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_registry_execute_all() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::new("tool1", "result1")));
        registry.register(Arc::new(MockTool::new("tool2", "result2")));

        let calls = vec![
            ToolCall {
                id: "1".to_string(),
                name: "tool1".to_string(),
                arguments: serde_json::json!({}),
            },
            ToolCall {
                id: "2".to_string(),
                name: "tool2".to_string(),
                arguments: serde_json::json!({}),
            },
        ];

        let results = registry.execute_all(&calls).await;
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }

    #[test]
    fn test_function_definitions() {
        let registry = ToolRegistry::with_defaults();
        let definitions = registry.to_function_definitions();

        assert!(!definitions.is_empty());
        for def in &definitions {
            assert!(!def.name.is_empty());
            assert!(!def.description.is_empty());
        }
    }

    // Property test: Tool call format
    #[test]
    fn test_tool_call_format_property() {
        use proptest::prelude::*;

        proptest!(|(id in "[a-z0-9]{8}", name in "[a-z_]{3,20}")| {
            let call = ToolCall {
                id: id.clone(),
                name: name.clone(),
                arguments: serde_json::json!({}),
            };

            prop_assert!(!call.id.is_empty());
            prop_assert!(!call.name.is_empty());
        });
    }

    /// **Feature: phase3-ai-agent-evolution, Property 11: Tool Output Feedback Loop**
    /// **Validates: Requirements 7.2**
    /// *For any* tool execution, the result should contain the original tool_call_id
    /// and tool name for proper correlation.
    #[test]
    fn test_tool_output_feedback_property() {
        use proptest::prelude::*;

        let rt = tokio::runtime::Runtime::new().unwrap();

        proptest!(|(
            call_id in "[a-z0-9]{8,16}",
            tool_name in "[a-z_]{3,15}",
            response in "[a-zA-Z0-9 ]{5,50}"
        )| {
            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(MockTool::new(tool_name.clone(), response.clone())));

            let call = ToolCall {
                id: call_id.clone(),
                name: tool_name.clone(),
                arguments: serde_json::json!({}),
            };

            let result = rt.block_on(registry.execute(&call));

            // Property: Result must contain original call_id for correlation
            prop_assert_eq!(&result.tool_call_id, &call_id);
            // Property: Result must contain tool name
            prop_assert_eq!(&result.name, &tool_name);
            // Property: Successful execution should have success=true
            prop_assert!(result.success);
            // Property: Content should match the tool response
            prop_assert_eq!(&result.content, &response);
        });
    }

    /// **Feature: phase3-ai-agent-evolution, Property 12: Tool Error Graceful Handling**
    /// **Validates: Requirements 7.3**
    /// *For any* tool execution failure, the system should return a ToolResult
    /// with success=false and an error message, not panic or propagate errors.
    #[test]
    fn test_tool_error_graceful_handling_property() {
        use proptest::prelude::*;

        let rt = tokio::runtime::Runtime::new().unwrap();

        proptest!(|(
            call_id in "[a-z0-9]{8,16}",
            tool_name in "[a-z_]{3,15}"
        )| {
            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(MockTool::failing(tool_name.clone())));

            let call = ToolCall {
                id: call_id.clone(),
                name: tool_name.clone(),
                arguments: serde_json::json!({}),
            };

            let result = rt.block_on(registry.execute(&call));

            // Property: Failed execution should have success=false
            prop_assert!(!result.success);
            // Property: Result must still contain original call_id
            prop_assert_eq!(&result.tool_call_id, &call_id);
            // Property: Result must still contain tool name
            prop_assert_eq!(&result.name, &tool_name);
            // Property: Error message should not be empty
            prop_assert!(!result.content.is_empty());
        });
    }

    /// **Feature: phase3-ai-agent-evolution, Property 12b: Tool Not Found Graceful Handling**
    /// **Validates: Requirements 7.3**
    /// *For any* tool call to a non-existent tool, the system should return
    /// a graceful error response, not panic.
    #[test]
    fn test_tool_not_found_graceful_property() {
        use proptest::prelude::*;

        let rt = tokio::runtime::Runtime::new().unwrap();

        proptest!(|(
            call_id in "[a-z0-9]{8,16}",
            tool_name in "[a-z_]{3,15}"
        )| {
            // Empty registry - no tools registered
            let registry = ToolRegistry::new();

            let call = ToolCall {
                id: call_id.clone(),
                name: tool_name.clone(),
                arguments: serde_json::json!({}),
            };

            let result = rt.block_on(registry.execute(&call));

            // Property: Non-existent tool should return failure, not panic
            prop_assert!(!result.success);
            // Property: Error message should indicate tool not found
            prop_assert!(result.content.contains("未找到") || result.content.contains("not found"));
        });
    }
}
