Improve web service — design
================================

Date: 2026-06-12
Author: Copilot + user

Summary
-------
Full refactor design to make the HTTP API server and background services (catalog refresher, tool updater, llmfit client) robust, observable, testable, and Windows-friendly. Key goals: eliminate crashes, provide automatic supervised restarts, standardize async runtime usage, introduce observability (tracing + metrics), and add a staged migration with tests and canary rollout guidance.

Goals & Success Criteria
------------------------
- No unexpected panics or process crashes from API or background tasks.
- Background jobs must be supervised and restarted on transient failure with exponential backoff.
- API responses remain low-latency; long blocking work runs off the async reactor (spawn_blocking).
- Health and readiness endpoints accurately reflect subsystem status.
- Structured logs + metrics available at /metrics for basic dashboards.
- Integration and e2e smoke tests covering health, menu, and i18n hot-swap.

High-level architecture
-----------------------
1. Single Tokio runtime (created in main) hosts all async components.
2. TaskManager (supervisor): spawns, monitors and optionally restarts named tasks. It tracks JoinHandles and restart state, applies backoff, and supports graceful shutdown.
3. Components run as managed tasks:
   - ApiServer (axum) — HTTP surface; handlers call ApiService (pure logic) which is async-friendly.
   - CatalogRefresher — periodically refreshes catalog using spawn_blocking for blocking HTML parsing and disk I/O.
   - ToolUpdater — periodic checks and updates; runs installs/updates in spawn_blocking.
   - LLMFit client / other long-polling clients — managed async tasks with circuit-breaker and retries.
4. Shared resources: WzllamaState, ArcSwap<I18n> (existing), telemetry registry (Prometheus), and tracing subscriber.

Components & Responsibilities
-----------------------------
- TaskManager
  - API: spawn(name, task_fn, restart_policy), stop(name), status(name)
  - Restart policy: disabled / always / on-failure with exponential backoff (configurable)
  - Monitor tasks, log failures, and restart according to policy
- ApiService (business logic)
  - Make functions async where needed; avoid blocking calls in handlers
  - Provide pure testable functions (no global runtime dependencies)
- ApiServer
  - Axum server with graceful shutdown signal integration to TaskManager
  - Expose endpoints: /health, /ready, /metrics, /api/menu, /api/tools, /api/menu/i18n
  - Protect long-running flows with timeouts and cancellation
- Background jobs
  - Implement periodic scheduling via tokio::time::interval in async tasks
  - Move heavy blocking work to spawn_blocking
  - Use client with timeouts/retries (reqwest with timeout + backoff)

Observability & Resiliency
--------------------------
- Logging: tracing with JSON output option; ensure consistent spans for requests and tasks
- Metrics: prometheus exporter; instrument:
  - API request counts & durations
  - Task restarts & failure counters
  - Catalog/tool update success/failure counts
- Health:
  - /health → basic process up
  - /ready → dependent on critical tasks (ApiServer: up; Ollama/llmfit optional but reflected)
- Retries & Circuit breakers:
  - Use tower::retry or custom retry with backoff for external HTTP calls
  - Apply a circuit-breaker for repeatedly failing external services
- Timeouts: define default timeouts for network & disk operations (configurable)

Error Handling
--------------
- Background tasks should never panic; wrap main loops with catch_unwind and report via TaskManager.
- Classify errors: transient vs permanent. Transient → retry with backoff. Permanent → log and mark task failed (no infinite restart loop).
- Persist minimal failure telemetry to disk (rolling logs) for post-mortem if required.

Testing Strategy
----------------
- Unit tests: ApiService functions, i18n reload, small parsing helpers.
- Integration tests: start TaskManager with ApiServer in test runtime, use reqwest to check /health, /api/menu, /api/menu/i18n hot-swap behavior.
- E2E smoke test: mock external endpoints (llmfit, docs.ollama) with local test servers; assert background tasks run and update caches.
- CI: include matrix runs for Linux and Windows (GitHub Actions) for integration tests.

Migration Plan (phased)
-----------------------
Phase A — Foundations (1–2 days)
- Add TaskManager type and wiring (no behavior change to current tasks).
- Introduce tracing subscriber and prometheus registry; expose /metrics in API server.
- Convert catalog refresher spawn to TaskManager (use spawn_blocking for fetch_and_update). Add timeouts and retries.
- Add unit tests for TaskManager behaviors (start/stop/restart).

Phase B — Background jobs (1–3 days)
- Migrate ToolUpdater and llmfit client to TaskManager tasks.
- Ensure all blocking work uses spawn_blocking and obeys timeouts.
- Add integration tests that simulate intermittent failures and verify restart/backoff.

Phase C — API & readiness (2–5 days)
- Refactor api_server::start_server to integrate with TaskManager and shared shutdown signal.
- Add readiness checks and ensure start_api_server_background integrates with the TaskManager lifecycle.
- Run e2e smoke tests and enable canary rollout steps.

Phase D — Polish & CI (1–2 days)
- Harden Windows-specific path handling and test packaging.
- Add dashboards/sample Prometheus+Grafana instructions.
- Finalize CI matrix, add scheduled integration runs.

Files likely changed
--------------------
- src/main.rs — create runtime + TaskManager initialization
- src/api_server.rs — refactor start_server to accept shutdown signals and metrics registry
- src/menu_api/api_service.rs — make async-safe where needed
- src/core/catalog_refresh.rs, src/core/tool_updater.rs, src/core/llmfit_api.rs — move to TaskManager tasks and use spawn_blocking
- src/config/i18n.rs — no major change; confirm subscribe() used by long-running tasks where appropriate
- tests/ — add integration / e2e tests

Estimates & Risks
-----------------
- Total: ~1–2 weeks depending on review and CI setup.
- Risks: interacting with blocking third-party CLIs/APIs, Windows path edge-cases, and complexity of graceful shutdown. Mitigation: incremental rollout and strong test doubles for external services.

Open questions / decisions
-------------------------
1. Restart policy default: restart-on-failure with capped exponential backoff? (recommended: yes)
2. Prometheus exposure: public /metrics or limited to localhost? (recommended: localhost by default)
3. CI matrix: include Windows runners for integration tests (recommended: yes)

Next steps (if approved)
------------------------
- Commit this design file to docs/superpowers/specs and push branch.
- Run the writing-plans skill to produce a concrete implementation plan with tasks, estimates, and the exact files/patches to change.

---
