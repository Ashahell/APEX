-- MCP Registries and Tools (persistence for dynamic discovery)
CREATE TABLE IF NOT EXISTS mcp_registries (
  id TEXT PRIMARY KEY,
  name TEXT
);

CREATE TABLE IF NOT EXISTS mcp_tools_registry (
  id TEXT PRIMARY KEY,
  registry_id TEXT,
  name TEXT,
  description TEXT,
  input_schema TEXT
);
