# s4-events

Event bus and pub/sub contracts.

## Responsibility

Decouple platform subsystems through typed domain events (import completed, graph updated, certificate issued).

## Public API

| Module | Purpose |
|--------|---------|
| `event` | `Event`, `EventKind` |
| `bus` | `EventBus` trait |
| `subscription` | `Subscription`, `EventHandler` |

## Tier

1 — Infrastructure
