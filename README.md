# pi-server

Experimental OpenCode-compatible HTTP API server backed by `pi --mode rpc` session processes.

`pi-server` is intended to let OpenCode clients attach to `pi` through the same headless server API exposed by `opencode serve`.

## Motivation

The goal is a native Rust server with low overhead:

- Fast startup in container / sandbox environments thanks to [pi_agent_rust](https://github.com/Dicklesworthstone/pi_agent_rust).
- Low resource usage compared with running a full JavaScript server stack.
- One `pi --mode rpc` process per active session, keeping backend state isolated.
- Compatibility with existing OpenCode clients, including `opencode attach`, `opencode run --attach`, and the desktop app.

This project is **experimental**. Expect compatibility gaps while the OpenCode API surface evolves.

## Usage

Build and run:

```sh
cargo run -- --hostname 127.0.0.1 --port 4096
```

Attach with OpenCode:

```sh
opencode attach http://127.0.0.1:4096
opencode run --attach http://127.0.0.1:4096 'tell me a joke'
```

Configuration:

- `PI_BIN_PATH`: path to the `pi` binary. Defaults to `~/.local/bin/pi`.
- `PI_SERVER_WORKDIR`: default working directory for new sessions. Defaults to the current directory.
- `--hostname`: bind host. Defaults to `127.0.0.1`.
- `--port`: bind port. Defaults to `4096`.

## Compatibility Matrix

| Area | Status | Notes |
| --- | --- | --- |
| `opencode attach` TUI | Working | Session creation, prompts, follow-ups, and event ordering covered by tests. |
| `opencode run --attach` | Working | Smoke test verifies assistant text streams through attached runs. |
| OpenCode desktop attach | Partial | Connects and chats; sidebar and live message event routing have regression coverage. More desktop UI paths may still need testing. |
| Session management | Working | Create, list, get, update, delete, fork, share/unshare compatibility shapes are covered. |
| Concurrent prompts | Working | Multiple sessions can prompt concurrently, each backed by its own `pi --mode rpc` process. |
| Live event stream | Working | User/assistant messages, text deltas, thinking, tool calls, session status, and directory-scoped desktop events are covered. |
| File/project/bootstrap routes | Partial | Enough shape compatibility for current clients; many routes are lightweight stubs. |
| Full `opencode serve` API parity | Incomplete | Route matrix is tracked, but not every endpoint has full behavior. |
| Persistence across server restarts | Not implemented | Sessions are currently in memory. |

## API Endpoint Status

Status legend:

- **Implemented**: backed by server state, `pi --mode rpc`, filesystem reads, or SSE behavior.
- **Partial**: returns OpenCode-compatible shapes, but behavior is incomplete.
- **Stub**: compatibility placeholder that returns an empty/no-op response.

### Core, Events, and Config

| Endpoint | Status | Notes |
| --- | --- | --- |
| `GET /doc` | Implemented | OpenAPI route document generated from the local route matrix. |
| `GET /global/health` | Implemented | Health response for attach/bootstrap. |
| `GET /global/event` | Implemented | Global SSE stream with directory-scoped events. |
| `GET /event` | Implemented | Instance-shaped SSE stream for TUI/CLI compatibility. |
| `POST /global/dispose` | Stub | No-op success response. |
| `POST /global/upgrade` | Stub | Returns current `pi-server` version. |
| `GET/PATCH /global/config` | Partial | Minimal config shape/update echo. |
| `GET/PATCH /config` | Partial | Minimal config shape/update echo. |
| `GET /config/providers` | Implemented | Minimal `pi` provider/model config. |
| `POST /instance/dispose` | Stub | No-op success response. |
| `POST /log` | Stub | No-op success response. |
| `GET /path` | Implemented | Returns directory/worktree paths, including directory query support. |

### Providers, Auth, Agents, and Tools

| Endpoint | Status | Notes |
| --- | --- | --- |
| `GET /provider` | Implemented | Minimal `pi` provider list. |
| `GET /provider/auth` | Stub | Empty auth state. |
| `POST /provider/{providerID}/oauth/authorize` | Stub | Empty response. |
| `POST /provider/{providerID}/oauth/callback` | Stub | No-op success response. |
| `GET /api/provider` | Implemented | v2 provider list shape. |
| `GET /api/provider/{providerID}` | Implemented | v2 provider detail shape. |
| `GET /api/model` | Implemented | v2 model list shape. |
| `PUT/DELETE /auth/{providerID}` | Stub | No-op success response. |
| `GET /agent` | Implemented | Minimal `build` agent. |
| `GET /command` | Stub | Empty command list. |
| `GET /skill` | Stub | Empty skill list. |
| `GET /experimental/tool` | Stub | Empty tool list. |
| `GET /experimental/tool/ids` | Stub | Empty tool ID list. |

### Sessions and Messages

| Endpoint | Status | Notes |
| --- | --- | --- |
| `GET /session` | Implemented | In-memory list with `directory`, `roots`, and `limit` filtering. |
| `POST /session` | Implemented | Creates a session and spawns `pi --mode rpc`; honors `x-opencode-directory`. |
| `GET /session/status` | Implemented | In-memory busy/idle status map. |
| `GET /session/{sessionID}` | Implemented | Returns session metadata. |
| `PATCH /session/{sessionID}` | Partial | Supports title and archived timestamp updates. |
| `DELETE /session/{sessionID}` | Implemented | Removes in-memory session. |
| `GET /session/{sessionID}/children` | Implemented | Lists forked child sessions. |
| `GET /session/{sessionID}/message` | Implemented | Lists stored messages/parts. |
| `POST /session/{sessionID}/message` | Implemented | Sends prompt to `pi` and records assistant response. |
| `GET /session/{sessionID}/message/{messageID}` | Implemented | Fetches a stored message by ID. |
| `DELETE /session/{sessionID}/message/{messageID}` | Stub | No-op success response. |
| `PATCH/DELETE /session/{sessionID}/message/{messageID}/part/{partID}` | Stub | Shape-compatible placeholder. |
| `POST /session/{sessionID}/prompt_async` | Implemented | Starts background prompt and publishes live events. |
| `POST /session/{sessionID}/command` | Partial | Converts command payload into a prompt. |
| `POST /session/{sessionID}/shell` | Partial | Converts shell command into a prompt. |
| `POST /session/{sessionID}/fork` | Partial | Creates an in-memory child session. |
| `POST /session/{sessionID}/abort` | Implemented | Calls RPC abort. |
| `POST/DELETE /session/{sessionID}/share` | Partial | Sets/clears local share metadata. |
| `POST /session/{sessionID}/revert` | Stub | Echoes session metadata. |
| `POST /session/{sessionID}/unrevert` | Stub | Echoes session metadata. |
| `POST /session/{sessionID}/init` | Stub | No-op success response. |
| `POST /session/{sessionID}/summarize` | Stub | No-op success response. |
| `GET /session/{sessionID}/todo` | Stub | Empty todo list. |
| `GET /session/{sessionID}/diff` | Stub | Empty diff list. |
| `POST /session/{sessionID}/permissions/{permissionID}` | Stub | No-op permission response. |
| `GET /api/session` | Partial | v2 list wrapper around in-memory sessions. |
| `GET /api/session/{sessionID}/message` | Partial | v2 message list wrapper. |
| `GET /api/session/{sessionID}/context` | Partial | Currently mirrors message list. |
| `POST /api/session/{sessionID}/prompt` | Implemented | v2 prompt wrapper. |
| `POST /api/session/{sessionID}/compact` | Stub | No content response. |
| `POST /api/session/{sessionID}/wait` | Stub | No content response. |

### Files, Search, VCS, and Projects

| Endpoint | Status | Notes |
| --- | --- | --- |
| `GET /file` | Implemented | Lists filesystem entries under server directory. |
| `GET /file/content` | Implemented | Reads text files under server directory. |
| `GET /file/status` | Stub | Empty file status list. |
| `GET /find` | Partial | Regex text search under server directory. |
| `GET /find/file` | Partial | Filename search under server directory. |
| `GET /find/symbol` | Stub | Empty symbol list. |
| `GET /vcs` | Partial | Minimal root/branch response. |
| `GET /vcs/status` | Stub | Empty status list. |
| `GET /vcs/diff` | Stub | Empty diff list. |
| `GET /vcs/diff/raw` | Stub | Empty text response. |
| `POST /vcs/apply` | Stub | No-op success response. |
| `GET /project` | Partial | Single project for server directory. |
| `GET /project/current` | Partial | Current project metadata. |
| `POST /project/git/init` | Stub | Returns current project metadata. |
| `PATCH /project/{projectID}` | Stub | Echoes update payload. |

### MCP, Permissions, Questions, PTY, Sync, and Experimental

| Endpoint | Status | Notes |
| --- | --- | --- |
| `GET/POST /mcp` | Stub | Empty MCP state. |
| `POST/DELETE /mcp/{name}/auth` | Stub | Placeholder auth start/remove responses. |
| `POST /mcp/{name}/auth/authenticate` | Stub | Empty response. |
| `POST /mcp/{name}/auth/callback` | Stub | Empty response. |
| `POST /mcp/{name}/connect` | Stub | No-op success response. |
| `POST /mcp/{name}/disconnect` | Stub | No-op success response. |
| `GET /permission` | Stub | Empty permission list. |
| `POST /permission/{requestID}/reply` | Stub | No-op success response. |
| `GET /question` | Stub | Empty question list. |
| `POST /question/{requestID}/reply` | Stub | No-op success response. |
| `POST /question/{requestID}/reject` | Stub | No-op success response. |
| `GET /lsp` | Stub | Empty LSP list. |
| `GET /formatter` | Stub | Empty formatter list. |
| `GET /pty/shells` | Partial | Static shell list. |
| `GET/POST /pty` | Stub | Empty list / fake PTY metadata. |
| `GET/PUT/DELETE /pty/{ptyID}` | Stub | Fake metadata / no-op delete. |
| `GET /pty/{ptyID}/connect` | Stub | No-op success response. |
| `POST /pty/{ptyID}/connect-token` | Stub | Fake token response. |
| `POST /sync/start` | Stub | No-op success response. |
| `POST /sync/replay` | Stub | Fake replay response. |
| `POST /sync/steal` | Stub | Echoes payload. |
| `POST /sync/history` | Stub | Empty history list. |
| `GET /experimental/console` | Stub | Empty console provider state. |
| `GET /experimental/console/orgs` | Stub | Empty org list. |
| `POST /experimental/console/switch` | Stub | No-op success response. |
| `GET /experimental/session` | Partial | Experimental wrapper over in-memory sessions. |
| `GET /experimental/resource` | Stub | Empty resource object. |
| `GET/POST/DELETE /experimental/worktree` | Stub | Current-directory placeholder / no-op delete. |
| `POST /experimental/worktree/reset` | Stub | No-op success response. |
| `GET /experimental/workspace` | Stub | Empty workspace list. |
| `POST /experimental/workspace` | Stub | Fake local workspace metadata. |
| `GET /experimental/workspace/adapter` | Stub | Empty adapter list. |
| `GET /experimental/workspace/status` | Stub | Empty status list. |
| `POST /experimental/workspace/sync-list` | Stub | No content response. |
| `POST /experimental/workspace/warp` | Stub | No content response. |
| `DELETE /experimental/workspace/{id}` | Stub | No-op success response. |

### TUI Control

| Endpoint | Status | Notes |
| --- | --- | --- |
| `POST /tui/append-prompt` | Stub | No-op success response. |
| `POST /tui/clear-prompt` | Stub | No-op success response. |
| `GET /tui/control/next` | Stub | Null response. |
| `POST /tui/control/response` | Stub | No-op success response. |
| `POST /tui/execute-command` | Stub | No-op success response. |
| `POST /tui/open-help` | Stub | No-op success response. |
| `POST /tui/open-models` | Stub | No-op success response. |
| `POST /tui/open-sessions` | Stub | No-op success response. |
| `POST /tui/open-themes` | Stub | No-op success response. |
| `POST /tui/publish` | Stub | No-op success response. |
| `POST /tui/select-session` | Stub | No-op success response. |
| `POST /tui/show-toast` | Stub | No-op success response. |
| `POST /tui/submit-prompt` | Stub | No-op success response. |

## Development

Compatibility work is test-driven in `tests/api_compat.rs`. When fixing a client issue, add a regression that uses the same route, query params, headers, and event shape as the real OpenCode client.

Run before handing off changes:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
git diff --check
```
