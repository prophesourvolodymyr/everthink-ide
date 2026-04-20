// tools/registry.rs — ToolRegistry: register and look up tools by name

use super::Tool;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry { tools: HashMap::new() }
    }

    /// Register a tool. Replaces any existing tool with the same name.
    pub fn register(&mut self, tool: impl Tool + 'static) {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
    }

    /// Look up a tool by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// All registered tool names.
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.tools.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Build the default registry with all built-in tools.
    pub fn default_registry() -> Self {
        let mut r = Self::new();
        r.register(super::bash::BashTool);
        r.register(super::fs::ReadTool);
        r.register(super::fs::WriteTool);
        r.register(super::fs::EditTool);
        r.register(super::fs::GlobTool);
        r.register(super::fs::GrepTool);
        r.register(super::web::WebFetchTool::new());
        r.register(super::verify::VerifyTool);
        r.register(super::mcp::McpStub);
        r
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
