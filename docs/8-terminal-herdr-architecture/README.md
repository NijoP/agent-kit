# Terminal/Herdr Architecture: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines how EAK integrates with Herdr (https://github.com/herdrdev/herdr) for persistent terminal sessions, workspaces, tabs, panes, PTY terminals, agent detection, attach/detach, session restore, agent-aware state, CLI/socket control, and remote operation.

**Critical**: Herdr is a **terminal/workspace runtime peripheral**. EAK owns the engineering state. Herdr provides the runtime boundary for agent execution and human interaction.

---

## Herdr Concepts (Studied from Herdr Repository)

| Concept | Description | EAK Integration |
|---------|-------------|-----------------|
| **Session** | Persistent terminal session with multiple workspaces | One EAK project = one Herdr session |
| **Workspace** | Named collection of tabs/panes | One engineering phase = one workspace (or tab) |
| **Tab** | Container for panes | One agent = one tab (or pane) |
| **Pane** | PTY terminal running a process | Agent CLI, kernel log tail, verification output |
| **PTY** | Pseudoterminal for process I/O | `eak` CLI, `herdr` agent processes, model provider logs |
| **Agent Detection** | Herdr detects which agent runs in which pane | EAK registers agent metadata with Herdr |
| **Attach/Detach** | User attaches to session from any client | Engineer attaches to see live agent output |
| **Session Restore** | Herdr restores PTY state on reconnect | EAK restores kernel state from event log |
| **Agent-Aware State** | Herdr tracks agent lifecycle | EAK phases map to Herdr panes |
| **CLI/Socket Control** | `herdr` CLI and Unix socket for control | EAK orchestrator controls Herdr via socket |
| **Remote Operation** | SSH/tunnel to remote Herdr daemon | EAK can run on remote machine, UI local |

---

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        EAK KERNEL                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  Orchestrator│  │  Phase FSMs │  │  Event Log  │             │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
│         │                │                │                     │
│         ▼                ▼                ▼                     │
│  ┌─────────────────────────────────────────────┐               │
│  │           Herdr Integration Adapter         │               │
│  │  (implements TerminalRuntime port trait)    │               │
│  └─────────────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        HERDR DAEMON                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  Session    │  │  Workspace  │  │  PTY Manager│             │
│  │  Manager    │  │  Manager    │  │             │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│         │                │                │                     │
│         ▼                ▼                ▼                     │
│  ┌─────────────────────────────────────────────┐               │
│  │              Unix Socket API                │               │
│  └─────────────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
       ┌────────────┐  ┌────────────┐  ┌────────────┐
       │  Tauri UI  │  │  CLI (eak) │  │  Remote    │
       │  (attach)  │  │  (attach)  │  │  SSH       │
       └────────────┘  └────────────┘  └────────────┘
```

---

## Port Trait (`eak-ports`)

```rust
#[async_trait]
pub trait TerminalRuntime: Send + Sync {
    /// Create a new session for a project
    async fn create_session(&self, project_id: EntityId, config: SessionConfig) -> Result<SessionId, TerminalError>;

    /// Attach to existing session (returns PTY info for client)
    async fn attach(&self, session_id: SessionId, client: ClientInfo) -> Result<AttachInfo, TerminalError>;

    /// Detach client
    async fn detach(&self, session_id: SessionId, client_id: ClientId) -> Result<(), TerminalError>;

    /// Spawn a pane running a command (agent, log tail, etc.)
    async fn spawn_pane(&self, session_id: SessionId, spec: PaneSpec) -> Result<PaneId, TerminalError>;

    /// Send input to pane
    async fn send_input(&self, pane_id: PaneId, data: Vec<u8>) -> Result<(), TerminalError>;

    /// Resize pane
    async fn resize(&self, pane_id: PaneId, cols: u16, rows: u16) -> Result<(), TerminalError>;

    /// Get pane output (for UI streaming)
    async fn subscribe_output(&self, pane_id: PaneId) -> Result<OutputStream, TerminalError>;

    /// Register agent metadata (for Herdr agent detection)
    async fn register_agent(&self, pane_id: PaneId, meta: AgentMetadata) -> Result<(), TerminalError>;

    /// List sessions
    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, TerminalError>;

    /// Save session state (Herdr handles PTY; we save kernel state)
    async fn save_session(&self, session_id: SessionId) -> Result<(), TerminalError>;

    /// Restore session (kernel replay + Herdr PTY restore)
    async fn restore_session(&self, session_id: SessionId) -> Result<(), TerminalError>;
}
```

---

## Session ↔ Project Mapping

| Herdr Concept | EAK Mapping |
|---------------|-------------|
| Session | Project (one-to-one) |
| Workspace | Engineering Phase (Requirement, Schematic, PCB, Verification, etc.) |
| Tab | Agent Instance (Lead, Component, Schematic, etc.) |
| Pane | Process: `eak run --phase <phase>`, `tail -f log`, `verification watch` |

### Session Config
```rust
struct SessionConfig {
    project_id: EntityId,
    name: String,                    // "EAK: <project_name>"
    workspaces: Vec<WorkspaceConfig>,
    env: HashMap<String, String>,    // API keys, paths
    cwd: PathBuf,                    // Project directory
}
```

### Workspace Config
```rust
struct WorkspaceConfig {
    name: String,                    // "Requirements", "Schematic", "PCB"
    tabs: Vec<TabConfig>,
}

struct TabConfig {
    name: String,                    // "Lead Agent", "Component Agent"
    panes: Vec<PaneSpec>,
}

struct PaneSpec {
    command: String,                 // "eak run --phase requirement --live"
    args: Vec<String>,
    agent_role: Option<AgentRole>,   // For agent detection
    working_dir: Option<PathBuf>,
}
```

---

## Agent Detection & Metadata

EAK registers agent metadata with Herdr so Herdr can:
- Display agent status in UI (running, waiting, error)
- Route logs to correct pane
- Enable agent-aware keybindings

```rust
struct AgentMetadata {
    role: AgentRole,                 // Lead, Research, Component, etc.
    phase_id: PhaseId,
    correlation_id: Uuid,            // Links to kernel event correlation
    model_provider: Option<String>,  // Which model this agent uses
    status: AgentStatus,             // Starting, Running, WaitingForProposal, WaitingForApproval, Done, Error
    started_at: DateTime<Utc>,
}
```

---

## Attach/Detach Flow

### Engineer Attaches (Tauri UI or CLI)
1. UI calls `TerminalRuntime::attach(session_id, client_info)`.
2. Herdr returns `AttachInfo` with PTY socket paths for each pane.
3. UI connects to PTY sockets → streams output, sends input.
4. Herdr tracks client count; session persists after all detach.

### Engineer Detaches
1. UI calls `TerminalRuntime::detach(session_id, client_id)`.
2. Herdr closes PTY sockets for that client.
3. Session continues running in background.

### Session Restore (After Crash/Reboot)
1. Herdr restores PTY processes (shells, running commands).
2. EAK kernel replays event log → reconstructs canonical model.
3. EAK orchestrator resumes phases from last `PhaseState`.
4. Agent panes re-attached; agents continue from FSM state.

---

## CLI/Socket Control

EAK orchestrator controls Herdr via Unix socket (or CLI for simple ops).

### Socket Protocol (JSON-RPC over Unix socket)
```json
// Request
{"jsonrpc": "2.0", "id": 1, "method": "session.create", "params": {"project_id": "..."}}

// Response
{"jsonrpc": "2.0", "id": 1, "result": {"session_id": "..."}}

// Notification (pane output)
{"jsonrpc": "2.0", "method": "pane.output", "params": {"pane_id": "...", "data": "base64..."}}
```

### CLI Commands (for human use)
```bash
herdr session create --project <id> --name "My Project"
herdr session list
herdr session attach <session_id>
herdr pane spawn <session_id> --command "eak run --phase schematic"
herdr agent register <pane_id> --role schematic --phase <phase_id>
```

---

## Remote Operation

### Scenario: Kernel on Build Server, UI on Laptop
1. Herdr daemon runs on build server (with kernel, KiCad, model providers).
2. Engineer SSH tunnels Herdr socket: `ssh -L /tmp/herdr.sock:/run/herdr.sock user@build-server`
3. Tauri UI connects to local socket → full terminal access.
4. All agent execution, KiCad, verification runs on server.
5. Only PTY streams cross network (low bandwidth, high latency tolerant).

### Security
- Herdr socket permissions: user-only (0600).
- SSH tunnel encrypts PTY traffic.
- Model provider credentials stay on server (never sent to client).

---

## Tauri UI Integration

### Terminal Component (`app/ui/src/components/Terminal.tsx`)
- Connects to Herdr PTY socket via WebSocket (Tauri plugin) or local proxy.
- Renders xterm.js for each pane.
- Tab bar shows workspaces (phases), tabs (agents), panes.
- Agent status badges from `AgentMetadata`.

### Session Management UI
- Session list (projects) with status.
- "New Session" → creates project + Herdr session.
- "Attach" → opens terminal tabs.
- "Detach" → closes UI, session persists.

---

## Herdr Adapter Implementation (`herdr` crate, planned)

### Responsibilities
1. **Socket Client** — Connects to Herdr daemon Unix socket.
2. **Protocol Codec** — JSON-RPC encode/decode.
3. **Session Lifecycle** — Create, attach, detach, list, save, restore.
4. **Pane Management** — Spawn, input, resize, output subscription.
5. **Agent Registration** — Map EAK phases to Herdr panes.
6. **Error Translation** — Herdr errors → `TerminalError`.

### Configuration
```toml
[terminal.herdr]
socket_path = "/run/herdr/socket"  # Or $XDG_RUNTIME_DIR/herdr/socket
cli_path = "herdr"                 # Fallback to CLI
auto_start_daemon = true
daemon_args = ["--log-level", "info"]
```

---

## Non-Goals (What Herdr Does NOT Do)

- **Own engineering state** — Kernel is sole authority.
- **Validate proposals** — Kernel capability seams do that.
- **Run verification** — Kernel engines do that.
- **Manage components** — EAK component intelligence does that.
- **Generate manufacturing outputs** — Kernel does that.

Herdr provides **only** the terminal/runtime boundary: PTY, sessions, attach/detach, remote.

---

## Next Document

[KiCad Integration Architecture →](../9-kicad-integration-architecture/README.md)