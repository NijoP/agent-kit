# EAK V1 — Multi-Model Development Infrastructure

> **Status:** Active  
> **Scope:** Permanent AI development infrastructure for EAK V1  
> **Last Updated:** 2026-08-23

---

## 1. Overview

This document describes the permanent multi-model AI development infrastructure for the Electronics Agent Kit (EAK) V1. The system provides:

- **Four specialized AI agents** with distinct roles and model assignments
- **Shared telemetry database** for cross-terminal usage tracking
- **Localhost dashboard** for real-time observability
- **Rate-limit handling** with shared provider state
- **Fresh-session persistence** — all configuration survives OpenCode restarts

---

## 2. Four-Model Architecture

### 2.1 Agent Definitions

| Agent | Model ID | Role | Primary Use Cases |
|-------|----------|------|-------------------|
| **EAK-Master** | `nvidia/nvidia/nemotron-3-ultra-550b-a55b` | Orchestrator | Coordination, delegation, final decisions |
| **EAK-Architect** | `nvidia/nvidia/nemotron-3-ultra-550b-a55b` | Architect | Deep reasoning, architecture, system design |
| **EAK-Engineer** | `nvidia/nvidia/nemotron-3-super-120b-a12b` | Engineer | Rust implementation, coding, refactoring |
| **EAK-Reviewer** | `nvidia/deepseek-ai/deepseek-v4-flash-0731` | Reviewer | Code review, verification, independent analysis |
| **EAK-Fast** | `nvidia/nvidia/nemotron-3.5-lightning-30b-a3b` | Fast/Inspector | Quick investigation, grep, file exploration |

### 2.2 Model Provider
All models use the **NVIDIA** provider (`enabled_providers: ["nvidia"]`).

### 2.3 Task Routing Policy

```
ARCHITECTURAL / EXTREMELY DIFFICULT
        → EAK-Architect (Nemotron 3 Ultra 550B)

NORMAL IMPLEMENTATION
        → EAK-Engineer (Nemotron 3 Super 120B)

INDEPENDENT REVIEW / CHALLENGE
        → EAK-Reviewer (DeepSeek V4 Flash 0731)

SIMPLE / FAST / REPETITIVE
        → EAK-Fast (Nemotron 3.5 Lightning 30B)
```

### 2.4 Recommended Workflow

For significant implementation work:

```
EAK-Architect (design)
       ↓
EAK-Engineer (implement)
       ↓
EAK-Reviewer (verify)
       ↓
EAK-Fast (validate/test)
       ↓
Tests pass
       ↓
EAK-Architect (if architectural decision needed)
```

---

## 3. Configuration Files

### 3.1 OpenCode Project Configuration
**File:** `/home/dev/electronics-agent-kit/opencode.json`

Contains:
- Default model: `nvidia/nvidia/nemotron-3-ultra-550b-a55b`
- Small model: `nvidia/nvidia/nemotron-3.5-lightning-30b-a3b`
- Default agent: `eak-master`
- Five agents: `eak-master`, `eak-architect`, `eak-engineer`, `eak-reviewer`, `eak-fast`
- Enabled providers: `["nvidia"]`

### 3.2 Workspace Integration
**File:** `/home/dev/electronics-agent-kit/eak/Cargo.toml`

Added `eak-telemetry` and `eak-dashboard` crates to workspace members.

---

## 4. Telemetry System

### 4.1 Database
**Location:** `data/telemetry/eak-telemetry.db` (SQLite with WAL mode)

**Tables:**
- `model_requests` — Individual request records with full metadata
- `provider_state` — Shared rate-limit state across all terminals
- `model_health` — Cached per-model health metrics
- `sessions` — Session summaries
- `daily_summaries` — Daily aggregated statistics

### 4.2 Tracked Metrics

**Per Request:**
- Timestamp, terminal ID, session ID
- Agent, model ID, provider
- Request type, HTTP status, latency
- Retry count, token usage (when available)
- Error category, rate-limit flag

**Per Model:**
- Total/successful/failed requests
- Rate-limit events, total retries
- Average latency, token usage
- Current status, cooldown state

**Per Provider:**
- Status (Available/RateLimited/Degraded/Offline)
- Rate-limit count, consecutive 429s
- Cooldown expiration, retry-after
- Max retries exhausted flag

**Global:**
- Total requests, tokens, error rate
- Active models, most/fastest used model

### 4.3 Rate-Limit Handling

**Shared State Mechanism:**
- All four OpenCode terminals share the same SQLite database
- `provider_state` table tracks NVIDIA provider status globally
- When any terminal observes HTTP 429:
  1. Records rate-limit event with timestamp
  2. Calculates cooldown (respects `Retry-After` header, else exponential backoff)
  3. Updates shared `provider_state` to `RateLimited`
  4. Other terminals see `RateLimited` status and defer requests

**Retry Policy:**
- Max retries: 3 (configurable)
- Base delay: 1 second
- Max delay: 60 seconds
- Jitter: ±20%
- Respects `Retry-After` header when present
- Stops after max retries exhausted → marks provider `Offline`

**Key Principle:** Never infinite retry loops. Bounded exponential backoff with hard limits.

---

## 5. Dashboard

### 5.1 Server
**Crate:** `eak-dashboard` (Axum + Tokio)
**Binary:** `eak-dashboard`
**Default Port:** 8080
**Default DB:** `data/telemetry/eak-telemetry.db`

### 5.2 API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /` | Dashboard HTML UI |
| `GET /api/health` | Service health check |
| `GET /api/models` | All model health metrics |
| `GET /api/global` | Global statistics |
| `GET /api/providers` | Provider state summary |
| `GET /api/requests/recent?limit=N` | Recent requests |
| `GET /api/requests/model/:model_id` | Requests for specific model |
| `GET /api/session/summary` | Session summaries |
| `GET /api/daily` | Daily summaries |

### 5.3 UI Features
- Real-time model status cards (4 models)
- Global statistics overview
- Provider state with cooldown timers
- Recent requests table with filtering
- Auto-refresh every 10 seconds
- Color-coded status badges (Healthy/Degraded/Rate Limited/Offline)

### 5.4 Management Script
**Script:** `./scripts/eak-dashboard {start|stop|status|restart}`

```bash
# Start dashboard
./scripts/eak-dashboard start

# Check status
./scripts/eak-dashboard status

# Stop dashboard
./scripts/eak-dashboard stop

# Restart dashboard
./scripts/eak-dashboard restart
```

---

## 6. Model Health Checking

### 6.1 Management Script
**Script:** `./scripts/eak-model-health {check|check-all|provider-state|model-health|test-one <agent>}`

```bash
# Verify config and model availability (no API calls)
./scripts/eak-model-health check

# Full check including live tests (uses quota!)
./scripts/eak-model-health check-all

# Show provider rate-limit state from DB
./scripts/eak-model-health provider-state

# Show model health metrics from DB
./scripts/eak-model-health model-health

# Test single agent (uses quota!)
./scripts/eak-model-health test-one eak-architect
```

### 6.2 Fresh Session Behavior
- Configuration loads automatically from `opencode.json`
- **No automatic live health checks** on session start
- Cached health data from telemetry DB shown immediately
- Live testing is explicit via `test-one` or `check-all`

---

## 7. Security

### 7.1 Credential Handling
- **No API keys in source code, config files, or Git**
- OpenCode manages credentials via its provider system
- Telemetry records **never** include raw credentials
- Only normalized error categories stored (e.g., `Authentication`, `RateLimit`)

### 7.2 Dashboard Security
- Binds to `127.0.0.1` (localhost) only
- No authentication required for local development
- No external network exposure

---

## 8. Fresh Session Persistence

### 8.1 What Persists
- `opencode.json` — Agent/model configuration
- `data/telemetry/eak-telemetry.db` — All historical telemetry
- Dashboard binary (built artifact)
- Documentation in `docs/engineering/`

### 8.2 Fresh Session Startup
```bash
cd /home/dev/electronics-agent-kit
opencode
```

New session automatically has:
- All 4 agents configured with correct models
- Access to telemetry database
- Dashboard available via `./scripts/eak-dashboard start`

### 8.3 Multi-Terminal Support
Each terminal gets unique `terminal_id` and `session_id` but shares:
- Same SQLite database (`data/telemetry/eak-telemetry.db`)
- Same provider state (rate-limit awareness)
- Same dashboard view

---

## 9. Known Limitations

### 9.1 Provider Quotas
- **NVIDIA provider limits: UNKNOWN / NOT EXPOSED**
- No official quota API discovered
- Rate-limit detection via HTTP 429 only
- Dashboard shows `UNKNOWN` for provider limits

### 9.2 DeepSeek Latency
- `deepseek-ai/deepseek-v4-flash-0731` observed significantly slower in OpenCode
- Treated as observable property, not a bug
- Telemetry records actual latency per model
- No automatic model substitution

### 9.3 Token Counting
- Token counts only available when provider returns them
- Many NVIDIA models return `null` for token usage
- Dashboard shows `Unknown` when unavailable
- **Never fabricates token counts**

### 9.4 Dashboard Persistence
- Dashboard is a separate binary, not integrated into OpenCode TUI
- Must be started explicitly via `./scripts/eak-dashboard start`
- Does not auto-start with OpenCode session

---

## 10. Commands Quick Reference

### Dashboard
```bash
./scripts/eak-dashboard start    # Start server on localhost:8080
./scripts/eak-dashboard status   # Check if running
./scripts/eak-dashboard stop     # Stop server
./scripts/eak-dashboard restart  # Restart server
```

### Model Health
```bash
./scripts/eak-model-health check           # Config + availability (no API)
./scripts/eak-model-health check-all       # Full live test (uses quota)
./scripts/eak-model-health provider-state  # DB provider state
./scripts/eak-model-health model-health    # DB model health
./scripts/eak-model-health test-one eak-architect  # Single model test
```

### Development
```bash
cd /home/dev/electronics-agent-kit/eak
cargo check --workspace          # Validate all crates
cargo test --workspace --exclude eak-cli  # Run tests
cargo build --release -p eak-dashboard    # Build dashboard
```

---

## 11. File Inventory

### Configuration
- `opencode.json` — OpenCode project config with 5 agents
- `eal/Cargo.toml` — Workspace with telemetry/dashboard crates

### Telemetry Crate (`eak/crates/eak-telemetry/`)
- `src/lib.rs` — Public API exports
- `src/models.rs` — Data structures (Request, Session, Health, ProviderState)
- `src/database.rs` — SQLite persistence layer
- `src/provider_state.rs` — Shared rate-limit state manager
- `src/retry.rs` — Bounded retry policy with backoff
- `src/telemetry.rs` — High-level recorder

### Dashboard Crate (`eak/crates/eak-dashboard/`)
- `src/main.rs` — Axum server entry point
- `src/routes.rs` — API endpoints
- `src/models.rs` — App state
- `static/index.html` — Dashboard UI

### Scripts
- `scripts/eak-dashboard` — Dashboard lifecycle management
- `scripts/eak-model-health` — Model health checking

### Documentation
- `docs/engineering/eak-multimodel-development.md` — This file
- `reports/eak-multimodel-opencode-validation.md` — Validation report

---

## 12. Maintenance

### Adding a New Model
1. Add model to `opencode.json` agents
2. Update `MODELS` array in `scripts/eak-model-health`
3. Update documentation tables
4. Test with `./scripts/eak-model-health test-one <new-agent>`

### Updating Rate-Limit Policy
Modify `RetryPolicy` defaults in `eak-telemetry/src/retry.rs`:
- `max_retries`
- `base_delay_ms`
- `max_delay_ms`
- `jitter_factor`

### Database Schema Changes
1. Add migration in `database.rs::init_schema()`
2. Handle schema version if needed
3. Update model structs in `models.rs`

---

## 13. Validation Checklist

- [x] Four models recognized by OpenCode
- [x] Four model roles documented
- [x] Multi-model configuration persists in `opencode.json`
- [x] Fresh OpenCode sessions inherit configuration
- [x] Multiple terminals can use infrastructure
- [x] Shared telemetry database works
- [x] Shared provider state works (rate-limit awareness)
- [x] Localhost dashboard works (HTTP API + UI)
- [x] Usage metrics tracked per model/session/day
- [x] Model health metrics tracked
- [x] Provider limits accurately represented as UNKNOWN
- [x] No API keys persisted anywhere
- [x] Documentation exists
- [x] `cargo check --workspace` passes
- [x] `cargo test --workspace --exclude eak-cli` passes

---

## 14. Changelog

| Date | Change |
|------|--------|
| 2026-08-23 | Initial multi-model infrastructure implementation |
| 2026-08-23 | Added eak-telemetry crate with SQLite persistence |
| 2026-08-23 | Added eak-dashboard crate with Axum server |
| 2026-08-23 | Created management scripts |
| 2026-08-23 | Documented architecture and operations |

---

*This document is the source of truth for EAK V1 multi-model development infrastructure. Update it whenever the architecture changes.*