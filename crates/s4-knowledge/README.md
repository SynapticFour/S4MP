# s4-knowledge

Software knowledge graph contracts — the core product model.

## Responsibility

Facts, provenance, confidence, lifecycle, and ontology extensions that turn code graphs into durable knowledge.

## Public API

| Module | Purpose |
|--------|---------|
| `extract` | `Concept`, `extract_concepts_from_graph`, `concepts_to_facts` (Proposed + heuristic confidence) |
| `fact` | `Fact`, `FactLifecycle`, `Confidence` |
| `provenance` | `Provenance`, `SourceType` |
| `ontology` | `Ontology`, extension kind registry |
| `materializer` | `KnowledgeMaterializer` trait (contract) |

## Tier

2 — Knowledge
