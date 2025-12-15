//! Tool Calling System Module
//!
//! Provides a framework for AI agents to call external tools.
//!
//! Features:
//! - Tool trait for defining custom tools
//! - Tool registry for managing available tools
//! - Built-in tools (weather, calculator, etc.)
//! - JSON Schema parameter validation

mod registry;

pub use registry::*;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use thiserror::Error;

/// Tool execution error
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Rate limited: retry after {0} seconds")]
    RateLimited(u64),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),
}

/// Tool trait - defines the interface for all tools
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool name (used for identification)
    fn name(&self) -> &str;

    /// Get a human-readable description
    fn description(&self) -> &str;

    /// Get the JSON Schema for parameters
    fn parameters(&self) -> Value;

    /// Execute the tool with given arguments
    async fn execute(&self, args: Value) -> Result<String, ToolError>;

    /// Check if the tool is available (e.g., API key configured)
    fn is_available(&self) -> bool {
        true
    }
}

/// Tool call request from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique ID for this tool call
    pub id: String,
    /// Tool name
    pub name: String,
    /// Arguments as JSON
    pub arguments: Value,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool call ID (matches ToolCall.id)
    pub tool_call_id: String,
    /// Tool name
    pub name: String,
    /// Result content (success or error message)
    pub content: String,
    /// Whether execution was successful
    pub success: bool,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(tool_call_id: impl Into<String>, name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            name: name.into(),
            content: content.into(),
            success: true,
        }
    }

    /// Create a failed result
    pub fn failure(tool_call_id: impl Into<String>, name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            name: name.into(),
            content: error.into(),
            success: false,
        }
    }
}

/// Function definition for LLM (OpenAI format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// Function description
    pub description: String,
    /// Parameters JSON Schema
    pub parameters: Value,
}

impl fmt::Display for FunctionDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.name, self.description)
    }
}

// ============== Built-in Tools ==============

/// Calculator tool for basic math operations
pub struct CalculatorTool;

impl CalculatorTool {
    pub fn new() -> Self {
        Self
    }

    fn evaluate(&self, expression: &str) -> Result<f64, ToolError> {
        // Simple expression evaluator
        // In production, use a proper math parser like `meval`
        let expr = expression.trim();

        // Handle simple operations
        if let Some((left, right)) = expr.split_once('+') {
            let a: f64 = left.trim().parse().map_err(|_| {
                ToolError::InvalidArguments(format!("Invalid number: {}", left))
            })?;
            let b: f64 = right.trim().parse().map_err(|_| {
                ToolError::InvalidArguments(format!("Invalid number: {}", right))
            })?;
            return Ok(a + b);
        }

        if let Some((left, right)) = expr.split_once('-') {
            if !left.is_empty() {
                let a: f64 = left.trim().parse().map_err(|_| {
                    ToolError::InvalidArguments(format!("Invalid number: {}", left))
                })?;
                let b: f64 = right.trim().parse().map_err(|_| {
                    ToolError::InvalidArguments(format!("Invalid number: {}", right))
                })?;
                return Ok(a - b);
            }
        }

        if let Some((left, right)) = expr.split_once('*') {
            let a: f64 = left.trim().parse().map_err(|_| {
                ToolError::InvalidArguments(format!("Invalid number: {}", left))
            })?;
            let b: f64 = right.trim().parse().map_err(|_| {
                ToolError::InvalidArguments(format!("Invalid number: {}", right))
            })?;
            return Ok(a * b);
        }

        if let Some((left, right)) = expr.split_once('/') {
            let a: f64 = left.trim().parse().map_err(|_| {
                ToolError::InvalidArguments(format!("Invalid number: {}", left))
            })?;
            let b: f64 = right.trim().parse().map_err(|_| {
                ToolError::InvalidArguments(format!("Invalid number: {}", right))
            })?;
            if b == 0.0 {
                return Err(ToolError::ExecutionFailed("Division by zero".to_string()));
            }
            return Ok(a / b);
        }

        // Try parsing as a single number
        expr.parse().map_err(|_| {
            ToolError::InvalidArguments(format!("Cannot evaluate expression: {}", expr))
        })
    }
}

impl Default for CalculatorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "执行基本数学运算（加、减、乘、除）"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "数学表达式，如 '2 + 3' 或 '10 / 2'"
                }
            },
            "required": ["expression"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let expression = args["expression"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'expression' parameter".to_string()))?;

        let result = self.evaluate(expression)?;
        Ok(format!("{} = {}", expression, result))
    }
}

/// Current time tool
pub struct CurrentTimeTool;

impl CurrentTimeTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CurrentTimeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CurrentTimeTool {
    fn name(&self) -> &str {
        "current_time"
    }

    fn description(&self) -> &str {
        "获取当前日期和时间"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "timezone": {
                    "type": "string",
                    "description": "时区，如 'Asia/Shanghai'，默认为 UTC"
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let now = chrono::Utc::now();

        let timezone = args["timezone"].as_str().unwrap_or("UTC");

        // For simplicity, just return UTC time with timezone note
        // In production, use chrono-tz for proper timezone handling
        if timezone == "UTC" {
            Ok(format!("当前时间 (UTC): {}", now.format("%Y-%m-%d %H:%M:%S")))
        } else {
            Ok(format!(
                "当前时间 (UTC): {} (请求时区: {})",
                now.format("%Y-%m-%d %H:%M:%S"),
                timezone
            ))
        }
    }
}

/// Weather tool (mock implementation)
pub struct WeatherTool {
    api_key: Option<String>,
}

impl WeatherTool {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }

    pub fn mock() -> Self {
        Self { api_key: None }
    }
}

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "获取指定城市的天气信息"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "城市名称，如：北京、上海、New York"
                }
            },
            "required": ["city"]
        })
    }

    fn is_available(&self) -> bool {
        // Mock is always available, real API needs key
        self.api_key.is_none() || self.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false)
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let city = args["city"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'city' parameter".to_string()))?;

        // Mock response for now
        // In production, call actual weather API
        if self.api_key.is_none() {
            // Return mock data
            Ok(format!(
                "{}的天气：晴，温度：25°C，湿度：60%，风速：10km/h",
                city
            ))
        } else {
            // Would call real API here
            Ok(format!(
                "{}的天气：晴，温度：25°C，湿度：60%，风速：10km/h",
                city
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculator_tool() {
        let tool = CalculatorTool::new();

        // Addition
        let result = tool.execute(serde_json::json!({"expression": "2 + 3"})).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("5"));

        // Subtraction
        let result = tool.execute(serde_json::json!({"expression": "10 - 4"})).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("6"));

        // Multiplication
        let result = tool.execute(serde_json::json!({"expression": "3 * 4"})).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("12"));

        // Division
        let result = tool.execute(serde_json::json!({"expression": "20 / 4"})).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("5"));

        // Division by zero
        let result = tool.execute(serde_json::json!({"expression": "10 / 0"})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_current_time_tool() {
        let tool = CurrentTimeTool::new();
        let result = tool.execute(serde_json::json!({})).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("当前时间"));
    }

    #[tokio::test]
    async fn test_weather_tool_mock() {
        let tool = WeatherTool::mock();
        let result = tool.execute(serde_json::json!({"city": "北京"})).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("北京"));
    }

    #[test]
    fn test_tool_result() {
        let success = ToolResult::success("call_1", "calculator", "2 + 2 = 4");
        assert!(success.success);
        assert_eq!(success.tool_call_id, "call_1");

        let failure = ToolResult::failure("call_2", "weather", "API error");
        assert!(!failure.success);
    }

    #[test]
    fn test_function_definition() {
        let tool = CalculatorTool::new();
        let def = FunctionDefinition {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            parameters: tool.parameters(),
        };

        assert_eq!(def.name, "calculator");
        assert!(!def.description.is_empty());
    }
}
