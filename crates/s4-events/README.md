# s4-events

Event bus and pub/sub contracts.

## Responsibility

Decouple platform subsystems through typed domain events (import completed, graph updated, certificate issued).

## Public API

| Module | Purpose |
|--------|---------|
| `event` | `Event`, `EventKind` |
| `memory` | **Live:** `RecordingEventSink` (sync; RFC-3339 timestamps) |
| `bus` | `EventBus` trait (contract; no impl yet) |
| `subscription` | `Subscription`, `EventHandler` (contract) |

## Tier

1 — Infrastructure
