# wzllama HTTP API

wzllama exposes a REST API on port **1133** for programmatic access to tools, models, and system information.

## Starting the API Server

```bash
wzllama serve
```

The server listens on `http://0.0.0.0:1133` by default.

## Authentication

Currently, the API has no authentication. If you need to expose it beyond localhost, consider using a reverse proxy with authentication.

## Endpoints

### Health Check

```http
GET /health
```

Returns `200 OK` with body `OK`.

### Tools

#### List All Tools

```http
GET /api/v1/tools
```

Returns an array of all available tools:

```json
[
  {
    "id": "ollama",
    "name": "Ollama",
    "description": "Local AI model server",
    "installed": true,
    "status": "installed",
    "supports_agentic": false,
    "requires_docker": false
  }
]
```

#### Get Tool Details

```http
GET /api/v1/tools/{tool_id}
```

Returns details for a specific tool:

```json
{
  "id": "ollama",
  "name": "Ollama",
  "description": "Local AI model server",
  "installed": true,
  "status": "installed",
  "supports_agentic": false,
  "requires_docker": false
}
```

#### Get Tool Status

```http
GET /api/v1/tools/{tool_id}/status
```

Returns installation status:

```json
{
  "id": "ollama",
  "installed": true,
  "status": "installed"
}
```

#### Install Tool

```http
POST /api/v1/tools/{tool_id}/install
```

Installs the specified tool:

```json
{
  "success": true,
  "message": "Ollama installed successfully"
}
```

#### Update Tool

```http
POST /api/v1/tools/{tool_id}/update
```

Updates the specified tool:

```json
{
  "success": true,
  "message": "Ollama updated successfully"
}
```

#### Uninstall Tool

```http
POST /api/v1/tools/{tool_id}/uninstall
```

Uninstalls the specified tool:

```json
{
  "success": true,
  "message": "Ollama uninstalled successfully"
}
```

#### Launch Tool

```http
POST /api/v1/tools/{tool_id}/launch
```

Returns information about launching interactively:

```json
{
  "success": true,
  "message": "To launch ollama interactively, use wzllama wizard or wzllama tools menu"
}
```

### Models

#### List Models

```http
GET /api/v1/models
```

Returns installed Ollama models:

```json
{
  "models": [
    {
      "name": "qwen2.5-coder:14b",
      "model": "qwen2.5-coder:14b",
      "size": 9500000000,
      "formatted_size": "9.5 GB"
    }
  ]
}
```

#### Pull Model

```http
POST /api/v1/models/{model_name}/pull
```

Returns command to pull model:

```json
{
  "success": true,
  "message": "To pull model qwen2.5-coder:14b, run: ollama pull qwen2.5-coder:14b"
}
```

#### Delete Model

```http
DELETE /api/v1/models/{model_name}
```

Returns command to delete model:

```json
{
  "success": true,
  "message": "To delete model qwen2.5-coder:14b, run: ollama rm qwen2.5-coder:14b"
}
```

### System

#### Hardware Info

```http
GET /api/v1/hardware
```

Returns system hardware information:

```json
{
  "ram_gb": 32.0,
  "has_gpu": true,
  "gpus": [
    {
      "name": "NVIDIA GeForce RTX 4090",
      "vram_mb": 24576
    }
  ]
}
```

#### System Status

```http
GET /api/v1/status
```

Returns system status:

```json
{
  "status": "running",
  "ollama": "connected"
}
```

### Menu

#### Get Menu Tree

```http
GET /api/v1/menu
```

Returns the menu structure:

```json
{
  "id": "root",
  "label": "wzllama",
  "children": [
    {
      "id": "wizard",
      "label": "Wizard",
      "action_id": "wizard"
    }
  ]
}
```

## Available Tools

| ID | Name | Description | Agentic | Docker |
|----|------|-------------|---------|--------|
| ollama | Ollama | Local AI model server | No | No |
| open_webui | Open WebUI | Web interface for AI models | No | Yes |
| openclaw | OpenClaw | Personal AI assistant | No | No |
| claude_code | Claude Code | Anthropic coding agent | Yes | No |
| hermes_agent | Hermes Agent | Self-improving AI agent | Yes | No |
| opencode | OpenCode | Open-source coding agent | Yes | No |
| codex | Codex | OpenAI coding agent | Yes | No |
| copilot_cli | Copilot CLI | GitHub AI agent | Yes | No |
| droid | Droid | Factory coding agent | Yes | No |
| pi | Pi | Minimal AI agent | Yes | No |
| pool | Pool | Poolside coding agent | Yes | No |
| obsidian | Obsidian | Note-taking app | No | No |
| goose | Goose | AI agent | Yes | No |
| llmfit | LLMFit | LLM training tool | No | No |

## Error Responses

All endpoints return appropriate HTTP status codes:

- `200 OK` - Success
- `400 Bad Request` - Invalid request
- `404 Not Found` - Tool or model not found
- `500 Internal Server Error` - Server error

Error response format:

```json
{
  "success": false,
  "message": "Error description"
}
```

## CORS

The API includes CORS headers allowing requests from any origin.