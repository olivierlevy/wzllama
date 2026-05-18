# MCP (Model Context Protocol) Configuration for LLMFit

## What is MCP?

MCP allows AI agents (Claude Desktop, Cursor, etc.) to discover and use tools over stdio. Each client spawns its own `llmfit serve --mcp` subprocess.

## Available Tools

When connected, LLMFit provides these MCP tools:

| Tool | Description | Parameters |
|------|-------------|------------|
| `get_system_specs` | Node hardware info (RAM, GPU, CPU) | None |
| `recommend_models` | Top models for this hardware | `limit?`, `use_case?`, `min_fit?`, `runtime?`, `license?`, `sort?` |
| `search_models` | Free-text model search | `query`, `limit?` |
| `plan_hardware` | Hardware requirements for a model | `model`, `context?`, `quant?`, `target_tps?` |
| `get_runtimes` | Installed inference runtimes | None |
| `get_installed_models` | Models in local runtimes | None |

## Configuration for Different Clients

### Claude Desktop

1. Install Claude Desktop from https://claude.ai/download
2. Open Claude Desktop Settings → Developer → Edit Config
3. Add the llmfit configuration to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "llmfit": {
      "command": "llmfit",
      "args": ["serve", "--mcp"]
    }
  }
}
```

4. Restart Claude Desktop

### Cursor

1. Open Cursor Settings → Features → MCP Servers
2. Add a new server with:
   - Name: `llmfit`
   - Command: `llmfit`
   - Args: `["serve", "--mcp"]`

### Windsurf

1. Open Windsurf Settings → AI → MCP Servers
2. Add the llmfit configuration

## Installing LLMFit First

Before using MCP, ensure LLMFit is installed:

```bash
uv tool install -U llmfit
```

Verify installation:
```bash
llmfit --version
```

## Alternative: HTTP Mode

For wzllama integration, LLMFit runs in HTTP mode:

```bash
llmfit serve --port 8787
```

This starts an HTTP API server that wzllama can query directly.

## Troubleshooting

### LLMFit not found

Make sure `~/.local/bin` is in your PATH:
```bash
export PATH="$HOME/.local/bin:$PATH"
```

### MCP tools not appearing in Claude

1. Check that llmfit is installed: `which llmfit`
2. Restart Claude Desktop completely
3. Check Claude logs: `~/Library/Logs/Claude/mcp.log` (macOS) or `%APPDATA%\Claude\mcp.log` (Windows)

### Connection issues

The MCP protocol uses stdio - no network configuration needed. Each client manages its own subprocess.