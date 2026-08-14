# s4-graph

Universal code graph contracts.

## Responsibility

Defines nodes, edges, graph views, layers, and query interfaces for the code graph.

## Public API

| Module | Purpose |
|--------|---------|
| `node` | `Node`, `NodeKind` |
| `edge` | `Edge`, `EdgeKind` |
| `view` | `GraphView`, `GraphBuilder` |
| `layer` | `GraphLayer` |
| `query` | `FilterQuery`, `GraphQuery`, `GraphDiff` |
| `memory` | `InMemoryGraph`, `InMemoryGraphView` (`GraphView::nodes()` enumerates sparse IDs) |

## Tier

1 — Knowledge (structural)
