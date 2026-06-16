# ADR: Sidecar Multi-Slot Architecture

| Status | Accepted (defer multi-process) |
| ------ | ------------------------------ |
| Date   | 2026-06-16                     |
| Tasks  | T008 spike, T012               |

## Context

SupportFlow runs one Python channel sidecar per desktop process via `ProcessHub`. T008 validated a second channel adapter (mock) behind the same contract, but `ProcessHub` still holds a **single active sidecar slot**.

Product need: multiple channels (e.g. wework + mock/dev) online simultaneously without RPC cross-talk.

## Decision

**Defer multi-process / multi-slot sidecar.** Keep **one active channel sidecar** per app instance for MVP.

| Option                                                              | Pros                                      | Cons                                                                             |
| ------------------------------------------------------------------- | ----------------------------------------- | -------------------------------------------------------------------------------- |
| **A. Multi-process sidecar** (`HashMap<channel_id, SidecarHandle>`) | True isolation, per-channel restart       | 2× memory, PyInstaller binaries, health/restart matrix, stdio routing complexity |
| **B. Single process, multi-channel router**                         | One binary, shared event loop             | Python SDK coupling; wework ntwork not designed for multi-tenant in one process  |
| **C. Single active channel (chosen)**                               | Matches current `ProcessHub`, lowest risk | Only one channel connected at a time                                             |

We choose **C** now. Option A remains the documented upgrade path when a second production channel must run concurrently.

## Consequences

### Product limitation

- Desktop runtime supports **one connected channel** at a time.
- Switching channels requires stopping the current sidecar before starting another (or app restart).
- T008 mock adapter proves the **Rust contract** is channel-agnostic; concurrency is an ops constraint, not a contract gap.

### Implementation (unchanged)

- `context/process_hub.rs` — single slot
- `python/sidecar/handler.rs` — RPC demux by method, not by channel instance
- Channel selection via config / `DEV_CHANNEL` env

### Future work (if revisiting A)

1. `ProcessHub` → `HashMap<String, SidecarSlot>` with channel_id key
2. Per-slot stdio pipes or named pipes; no shared RPC id space across slots
3. Health probe + exponential backoff restart per slot
4. Integration test: two mock channels RPC in parallel without id collision

## References

- [`plan/rust-sidecar-async-ipc-architecture.md`](../plan/rust-sidecar-async-ipc-architecture.md)
- [`docs/channel-adapter-contract.md`](./channel-adapter-contract.md)
- T008 spike notes: [`docs/channel-second-adapter-spike.md`](./channel-second-adapter-spike.md)
