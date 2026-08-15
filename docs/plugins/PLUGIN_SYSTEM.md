# S4MP Plugin System
## Architecture Specification v0.1

> **Status:** Target design. Shipped code is an **in-process registry of first-party frontends** (`s4 plugin list`), not a loader, sandbox, or third-party SDK.
> **Contract crate:** `s4-plugin` (manifests + host trait)
> **Shipped:** Java/Rust Tree-sitter frontends dispatched by language id in `s4-parser` (`extract_for_language`). WASM deferred: [ADR-016](../adr/0016-phase6-in-process-host-wasm-deferred.md).
> **Principle:** Do not list a plugin in the host unless `graph build` / `reason` actually uses it.

---

## 1. Purpose

The S4MP plugin system is the **extension bus** for all volatile platform behavior. Language parsing, metrics, LLM providers, import/export, verification, and analysis are plugins — not core dependencies.

| Goal | Mechanism |
|------|-----------|
| Core isolation | Artifact-based I/O; trait objects / WASM exports only |
| Third-party ecosystem | Signed manifests, sandbox, capability permissions |
| LLM interchangeability | `LlmProvider` plugin trait; core uses `s4-llm` contracts |
| Long-term stability | Frozen `s4-plugin` API semver; implementations version independently |
| Auditability | Every plugin output tagged with provenance + plugin identity |

**Why:** Platforms that embed parsers and LLM SDKs in core die when languages and vendors change. Plugins are how S4MP survives five years.

---

## 2. Architectural Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│  SURFACES: s4-cli · s4-api · s4-ui                          │
├─────────────────────────────────────────────────────────────┤
│  ORCHESTRATION: pipelines · jobs · workspace                │
├─────────────────────────────────────────────────────────────┤
│  PLUGIN HOST: load · sandbox · invoke · registry            │
├─────────────────────────────────────────────────────────────┤
│  PLUGIN API (s4-plugin): traits · manifest · context        │
╞═════════════════════════════════════════════════════════════╡
│  PLUGINS (implementations — core never imports these)       │
│  parsers · metrics · llm · importers · exporters ·          │
│  verifiers · analyzers                                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    s4-storage (CAS artifacts)
```

**Forbidden:**
- Core crates (`s4-core` … `s4-graph`) → plugin implementation crates
- Core crates → LLM SDKs, language parser generators, HTTP clients
- Plugins → other plugins directly (host mediates)
- Plugins → mutable graph memory in production path

---

## 3. Plugin Taxonomy

Every plugin implements **`Plugin`** (base). Specialized plugins implement **one primary role trait** (+ optional secondary traits).

| Role | Trait | Input artifacts | Output artifacts |
|------|-------|-----------------|------------------|
| **Importer** | `Importer` | URI, credentials ref | Physical snapshot, file tree |
| **Language parser** | `Parser` | File blob, language hint | Syntax tree, USIR module |
| **Linker** | `Linker` | USIR modules | Unified USIR, symbol table |
| **Analysis engine** | `Analyzer` | Graph slice, USIR | Findings, architecture views |
| **Metrics** | `MetricProvider` | Graph slice, symbols | Metric nodes, aggregates |
| **LLM provider** | `LlmProvider` | Context bundle, request | Proposal artifacts (proposed lifecycle) |
| **Exporter** | `Exporter` | Graph, knowledge slice | External format blob |
| **Verification engine** | `Verifier` | Graph, invariants, rules | Verification result, certificates |

### 3.1 Role Trait Summary

```rust
// Conceptual — defined in s4-plugin

trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    fn api_version(&self) -> ApiVersion;
    fn health_check(&self, ctx: &HostContext) -> Result<()>;
}

trait Importer: Plugin {
    fn import(&self, ctx: &mut InvokeContext, request: ImportRequest) -> Result<()>;
}

trait Parser: Plugin {
    fn parse(&self, ctx: &mut InvokeContext, unit: ParseUnit) -> Result<()>;
}

trait Linker: Plugin {
    fn link(&self, ctx: &mut InvokeContext, request: LinkRequest) -> Result<()>;
}

trait Analyzer: Plugin {
    fn analyze(&self, ctx: &mut InvokeContext, request: AnalysisRequest) -> Result<()>;
}

trait MetricProvider: Plugin {
    fn collect(&self, ctx: &mut InvokeContext, request: MetricRequest) -> Result<()>;
}

trait LlmProvider: Plugin {
    fn reason(&self, ctx: &mut InvokeContext, request: ReasonRequest) -> Result<()>;
}

trait Exporter: Plugin {
    fn export(&self, ctx: &mut InvokeContext, request: ExportRequest) -> Result<()>;
}

trait Verifier: Plugin {
    fn verify(&self, ctx: &mut InvokeContext, request: VerifyRequest) -> Result<()>;
}
```

**Why separate `MetricProvider` from `Analyzer`:** Metrics are high-volume, deterministic, and cache-friendly. Analysis includes heuristics and architecture extraction. Different invalidation, scaling, and sandbox policies.

**Why `LlmProvider` aligns with `s4-llm`:** The `ReasonRequest` / `Proposal` types live in `s4-llm`. Plugins implement transport to OpenAI, Anthropic, Ollama, etc.

---

## 4. Invocation Model

### 4.1 Artifact-Only I/O

Plugins **never** receive file paths to arbitrary disk, raw graph pointers, or network handles unless explicitly granted.

```
InvokeContext {
  host:           HostServices      // injected capabilities
  inputs:         &[ArtifactId]
  outputs:        &mut Vec<ArtifactId>
  diagnostics:    &mut Vec<Diagnostic>
  cancellation:   CancellationToken
}
```

Flow:
1. Host writes input artifacts to store (or references existing)
2. Host constructs `InvokeContext` with injected services
3. Host calls plugin trait method
4. Plugin reads via `host.store().read(id)`, writes via `host.store().write(artifact)`
5. Plugin records output IDs in `ctx.outputs`
6. Host attaches provenance: `{ plugin_id, plugin_version, api_version }`

### 4.2 HostServices (Dependency Injection Surface)

The host injects **only what the plugin declared** in its manifest permissions.

```rust
trait HostServices {
    fn store(&self) -> &dyn StoreReader;           // always (read)
    fn store_mut(&self) -> Option<&mut dyn StoreWriter>; // if write granted
    fn config(&self) -> &PluginConfig;             // plugin-scoped config
    fn events(&self) -> Option<&dyn EventBus>;     // if publish granted
    fn secrets(&self) -> Option<&dyn SecretProvider>; // if secrets granted
    fn temp(&self) -> &dyn TempDir;                // scratch space (quota)
    fn log(&self) -> &dyn PluginLogger;
}
```

**Why DI via host:** Core controls capability exposure. Third-party plugins get read-only store + temp dir; trusted parsers get write + no network; LLM plugins get secrets + network.

---

## 5. Plugin Manifest

Every plugin ships `s4-plugin.toml` (or embedded JSON in WASM custom section).

```toml
[plugin]
name = "s4-parser-rust"
version = "1.2.0"
api_version = "0.1"
description = "Rust USIR parser"
authors = ["SynapticFour"]
license = "MIT OR Apache-2.0"

[capabilities]
roles = ["parser"]
languages = ["rust"]
file_patterns = ["**/*.rs"]

[permissions]
store_read = true
store_write = true
network = false
secrets = false
env_vars = []
max_temp_bytes = 104857600  # 100 MiB

[compatibility]
min_host_version = "0.1.0"
max_host_version = "0.x"

[dependencies]
plugins = []  # optional plugin-to-plugin version constraints

[config.schema]
# JSON Schema for plugin-specific config section
```

Extended manifest fields:

| Field | Purpose |
|-------|---------|
| `roles` | One or more role traits implemented |
| `permissions` | Sandbox policy inputs |
| `compatibility` | Host version range |
| `signature` | Ed25519 signature over manifest + artifact hash |
| `artifact` | Content hash of plugin binary / WASM module |

---

## 6. Dynamic Loading vs Static Registration

Both modes coexist. The **same trait API** applies; only the linking mechanism differs.

### 6.1 Comparison

| Aspect | Static registration | Dynamic loading |
|--------|---------------------|-----------------|
| **Link time** | Compile-time (`inventory`, feature flags) | Runtime (`libloading`, WASM) |
| **Use case** | First-party, trusted, max performance | Third-party, optional, hot-reload |
| **Distribution** | Linked into `s4-cli` / host binary | `.so`, `.dylib`, `.dll`, `.wasm` |
| **Sandbox** | In-process (trust tier 1) | WASM preferred (tier 2–3) |
| **Upgrade** | Rebuild host | Drop-in artifact swap |
| **Discovery** | `build.rs` registry embed | Filesystem / registry download |

### 6.2 Static Registration (Phase 1 — Trusted)

```
build time:
  plugins/s4-parser-rust → linked via workspace dependency
  inventory::submit!(ParserRegistration { factory: ... })

runtime:
  PluginRegistry::built_in() → Vec<Box<dyn Parser>>
```

**Recommended crates:**
- [`linkme`](https://crates.io/crates/linkme) or [`inventory`](https://crates.io/crates/inventory) — distributed slice registration
- Workspace `members` under `plugins/`

**Why start here:** Fastest path to MVP; SynapticFour-controlled parsers ship in-process.

### 6.3 Dynamic Loading (Phase 2 — Ecosystem)

**Native dynamic libraries:**
```
PluginArtifact {
  format: "s4-native-v1"
  library_path: Path
  entry_symbol: "s4_plugin_create"   // C ABI
  destroy_symbol: "s4_plugin_destroy"
}
```

**Recommended crates:**
- [`libloading`](https://crates.io/crates/libloading) — dlopen
- [`abi_stable`](https://crates.io/crates/abi_stable) — safe-ish Rust ABI across versions (evaluate vs minimal C ABI)

**WASM modules (preferred for third-party):**
```
PluginArtifact {
  format: "s4-wasm-v1"
  module: ArtifactId           // WASM bytes in CAS
  exports: ["s4_plugin_init", "s4_invoke"]
}
```

**Recommended crates:**
- [`wasmtime`](https://crates.io/crates/wasmtime) — sandboxed runtime
- [`wit-bindgen`](https://crates.io/crates/wit-bindgen) — WIT interface generation (Component Model)
- [`wasmparser`](https://crates.io/crates/wasmparser) — validate before load

### 6.4 Decision Matrix

| Trust tier | Loading | Sandbox |
|------------|---------|---------|
| **T1 — First-party** | Static or native dynamic | In-process, no sandbox |
| **T2 — Signed partner** | WASM or native + seccomp | WASM default |
| **T3 — Community** | WASM only | No network unless declared + approved |

**Rule:** Third-party plugins **never** ship as unsandboxed native code in default configuration.

---

## 7. Security

### 7.1 Threat Model

| Threat | Mitigation |
|--------|------------|
| Malicious plugin reads secrets | SecretProvider returns scoped refs; WASM has no host FS |
| Data exfiltration via network | Network permission opt-in; WASM firewall |
| Store corruption | Append-only CAS; plugins get write, not mutate |
| Supply chain tampering | Signed manifests; hash-pinned lockfile |
| Resource exhaustion | Timeouts, memory limits, output size caps |
| Privilege escalation | No raw pointers across boundary; capability manifest |

### 7.2 Signing & Provenance

```
PluginPackage {
  manifest:     PluginManifest
  artifact:     ArtifactId          // binary or WASM
  signature:    Ed25519Signature    // over (manifest || artifact_hash)
  publisher:    PublisherId
}
```

**Recommended crates:**
- [`ed25519-dalek`](https://crates.io/crates/ed25519-dalek) — manifest signing
- [`coset`](https://crates.io/crates/coset) — optional COSE wrappers for registry transport

**Trust roots:**
- SynapticFour root key (first-party)
- Curated publisher keys (marketplace)
- User-local `trust-keys/` (explicit opt-in)

### 7.3 Permission Enforcement

Permissions declared in manifest are **maximum**; workspace config can **restrict further**, never expand.

```
effective_permissions = manifest.permissions ∩ workspace.policy ∩ trust_tier.defaults
```

LLM plugins: `network = true`, `secrets = ["OPENAI_API_KEY"]` — host injects secret by name, never environment dump.

---

## 8. Sandboxing

### 8.1 WASM Sandbox (Primary)

| Resource | Limit |
|----------|-------|
| Memory | 256 MB default (configurable) |
| CPU | Fuel / epoch interruption (Wasmtime) |
| Wall time | 30s default per invocation |
| Output size | 50 MB artifact cap |
| Network | Host-side HTTP proxy only — plugin calls `host.http(request)` |
| Filesystem | None — temp via `host.temp()` pre-opened FDs |

**WIT interface (conceptual):**
```wit
package s4:plugin@0.1.0;

interface host {
  store-read: func(id: string) -> result<list<u8>, error>;
  store-write: func(data: list<u8>) -> result<string, error>;
  log: func(level: log-level, message: string);
  http-post: func(url: string, headers: list<tuple<string,string>>, body: list<u8>) -> result<http-response, error>;
}

interface parser {
  parse: func(inputs: list<string>) -> result<list<string>, error>;
}
```

**Why WIT/Component Model:** Language-agnostic plugin authoring (Rust, C, future Go) with explicit host imports.

### 8.2 Native Sandbox (Fallback — T2 only)

For performance-critical first-party parsers that cannot tolerate WASM overhead:

- [`landlock`](https://crates.io/crates/landlock) — Linux filesystem sandbox
- [`seccompiler`](https://crates.io/crates/seccompiler) — seccomp BPF (Linux)
- [`sandboxer`](https://crates.io/crates/sandboxer) — evaluate maturity
- macOS: `sandbox-exec` profiles (platform-specific)
- Process isolation: separate worker process + IPC (heavier but robust)

**Architecture:** Native sandbox runs in **worker pool**; host serializes `InvokeContext` over IPC (cap'n proto / postcard).

### 8.3 Sandbox Selection Flow

```
load(plugin) → read trust_tier
  T1 → in_process(factory)
  T2 → if wasm_available { wasm } else { native_worker_sandbox }
  T3 → wasm_only or reject
```

---

## 9. Version Compatibility

Three independent version axes:

| Axis | Format | Rule |
|------|--------|------|
| **Plugin API** | `0.1`, `0.2` … | Frozen in `s4-plugin`; major bump = breaking trait change |
| **Plugin package** | Semver `1.2.0` | Independent release cycle |
| **Host platform** | Semver `0.1.0` | Manifest `min_host_version` / `max_host_version` |

### 9.1 Compatibility Check (Load Time)

```
compatible =
  plugin.api_version.major == host.api_version.major
  AND plugin.api_version.minor <= host.api_version.minor
  AND host.version satisfies plugin.compatibility range
```

Failure → structured error with remediation (`upgrade host`, `pin plugin version`).

**Recommended crate:** [`semver`](https://crates.io/crates/semver)

### 9.2 ABI Stability (Dynamic Native)

- **C ABI vtable** for `PluginV1` — never change layout; add `PluginV2` parallel export
- Plugins compiled against `s4-plugin-sdk` version X run on host SDK X–Y
- `s4_plugin_api_version()` exported symbol checked before `create`

### 9.3 WASM Interface Versioning

- WIT package `s4:plugin@0.1.0` — bump package version on breaking host import change
- WASM module declares required WIT versions in custom section
- Host supports N and N-1 WIT packages during transition

### 9.4 Workspace Lockfile

```toml
# s4.lock (generated)
[[plugins]]
name = "s4-parser-rust"
version = "1.2.0"
artifact = "blake3:abc123..."
api_version = "0.1"
trust = "first-party"
```

Reproducible builds and certification require **pinned artifact hashes**, not just semver ranges.

---

## 10. Plugin Discovery

### 10.1 Discovery Sources (Priority Order)

| Source | Scope | Mechanism |
|--------|-------|-----------|
| **Built-in registry** | Statically linked plugins | Compile-time registration |
| **Workspace config** | Project-enabled plugins | `s4.toml` `[plugins]` table |
| **Local plugin dir** | Developer overrides | `.s4mp/plugins/` or `~/.s4/plugins/` |
| **Project vendor dir** | Vendored plugins | `vendor/plugins/` |
| **Remote registry** | Ecosystem (future) | HTTPS registry API |

### 10.2 Resolution Algorithm

```
1. Load workspace s4.toml plugin declarations
2. Load s4.lock pins (must resolve exactly)
3. Merge built-in plugins (fill defaults for parser, importer)
4. Scan local dirs (dev mode only unless explicitly enabled)
5. Fetch remote (if name not found locally) → verify signature
6. Validate manifest permissions against workspace policy
7. Register in PluginRegistry keyed by (role, language, pattern)
```

### 10.3 Capability Routing

When pipeline needs a parser for `*.rs`:

```
registry.resolve(ResolveQuery {
  role: Parser,
  language: Some("rust"),
  file_pattern: "**/*.rs",
}) → PluginHandle
```

Multiple matches → workspace explicit priority list; else highest trust tier wins.

### 10.4 Recommended Crates

| Crate | Role |
|-------|------|
| [`globset`](https://crates.io/crates/globset) | File pattern matching |
| [`reqwest`](https://crates.io/crates/reqwest) | Remote registry (host only, not plugins) |
| [`serde_json`](https://crates.io/crates/serde_json) | Manifest parsing |

---

## 11. Configuration

### 11.1 Configuration Layers

```
defaults (platform)
  ↓ overridden by
workspace s4.toml
  ↓ overridden by
environment variables (S4_PLUGIN_<NAME>_*)
  ↓ overridden by
CLI flags (--plugin.rust.opt-level=2)
  ↓ secrets from
SecretProvider (never in toml on disk)
```

### 11.2 Workspace Plugin Config

```toml
# s4.toml
[plugins.parser]
default = "s4-parser-rust"

[plugins.parser.rust]
enabled = true
# plugin-specific config validated against manifest config.schema
[plugins.parser.rust.options]
include_tests = false

[plugins.llm]
default = "s4-llm-openai-compatible"

[plugins.llm.openai]
model = "gpt-4o"
# api_key = use SecretProvider: OPENAI_API_KEY

[plugins.metrics]
enabled = ["complexity", "coupling"]

[plugins.policy]
allow_network = ["s4-llm-*"]
max_trust_tier = "signed"
```

### 11.3 Config Injection

Host validates plugin config against JSON Schema from manifest, then passes frozen `PluginConfig` arc into `InvokeContext`.

Plugins **must not** read process environment directly (WASM cannot); they call `ctx.host.config().get("model")`.

---

## 12. Dependency Injection (Detailed)

### 12.1 Principles

| Principle | Implementation |
|-----------|----------------|
| **Constructor injection** | Host creates plugin instance with `PluginFactory::create(host_services)` |
| **Interface segregation** | Plugins see narrow `HostServices`, not whole platform |
| **No service locator** | No global `get_store()` singleton |
| **Request-scoped context** | Fresh `InvokeContext` per invocation |
| **Test doubles** | `s4-plugin-sdk` provides `MockHostServices` |

### 12.2 Factory Pattern

```rust
trait PluginFactory: Send + Sync {
    fn create(&self, services: HostServices) -> Result<Box<dyn Plugin>>;
    fn manifest(&self) -> &PluginManifest;
}

// Static registration
inventory::submit! {
    PluginFactoryRegistration {
        role: Role::Parser,
        factory: || Box::new(RustParserFactory),
    }
}
```

### 12.3 Pipeline Composition (Orchestrator DI)

The pipeline crate receives `PluginRegistry` + `HostServices` — it does not construct plugins:

```
PipelineExecutor {
  registry: Arc<PluginRegistry>,    // injected
  host: Arc<HostServices>,          // injected
  store: Arc<dyn Store>,            // injected
}
```

Stage dispatch:
```
ImportStage  → registry.resolve(Importer) → invoke
ParseStage   → registry.resolve(Parser)   → invoke (per file)
MetricsStage → registry.resolve(MetricProvider) → invoke
```

**Why:** Core orchestration depends on traits; registry resolves implementations at runtime from config.

### 12.4 Plugin-to-Plugin Dependencies

Plugins **must not** call each other directly.

If parser B requires linker output format from linker A:
- Declare `[dependencies.plugins]` in manifest for **version resolution only**
- Host orchestrates: linker stage → parser stage via artifact IDs

Exception: **plugin bundles** (single publisher ships parser+linker as one package exporting multiple roles) — still one manifest, multiple role exports.

---

## 13. Plugin Lifecycle

```
Discover → Validate → Load → Register → HealthCheck
    → Invoke* → Unload (dynamic) / ProcessExit (static)
```

| State | Description |
|-------|-------------|
| `Discovered` | Manifest found, not validated |
| `Validated` | Signature, API version, permissions OK |
| `Loaded` | Binary/WASM mapped in memory |
| `Registered` | Available in registry for resolution |
| `Disabled` | Config-disabled; skip resolution |
| `Failed` | Load/health error; diagnostic recorded |

**Health check:** Lightweight call on register and periodically (configurable). Failed → mark disabled, emit `s4-events` alert.

---

## 14. Error Handling & Diagnostics

```
Diagnostic {
  level:    info | warn | error
  code:     string          // e.g. "parse/unresolved-import"
  message:  string
  location: Option<SourceLocation>
  artifact: Option<ArtifactId>
}
```

Plugin errors **do not panic the host**. Return `Result<PluginOutput, PluginError>`.

Structured error codes enable CI gates: `verify/traceability/incomplete` → fail build.

---

## 15. Testing & Conformance

### 15.1 Conformance Suite (`s4-plugin-conformance`, future)

| Suite | Validates |
|-------|-----------|
| `parser-base` | Emits valid USIR artifact schema |
| `metric-base` | Emits valid metric nodes |
| `llm-base` | Proposals always `proposed` lifecycle |
| `verifier-base` | Emits verification artifact schema |

Plugins publish `cargo test -p s4-plugin-conformance -- --plugin wasm` before registry submission.

### 15.2 SDK (`s4-plugin-sdk`)

- `MockHostServices`, `MockStore`
- Test fixture artifacts
- `invoke_plugin()` test harness
- Proc macros for manifest embedding

---

## 16. Crate Map

| Crate | Responsibility |
|-------|----------------|
| `s4-plugin` | Traits, manifest types, InvokeContext, permissions (stable API) |
| `s4-plugin-sdk` | Plugin author helpers, mocks, test harness |
| `s4-plugin-host` | Load, sandbox, invoke, factory registry |
| `s4-plugin-registry` | Discovery, resolution, lockfile, remote fetch |
| `s4-llm` | ReasonRequest/Proposal types (LLM plugins implement via trait) |
| `s4-storage` | CAS (injected into host) |
| `s4-events` | Plugin lifecycle events |
| `s4-project` | Workspace plugin config + lockfile |

---

## 17. Phased Delivery

| Phase | Deliverable |
|-------|-------------|
| **P1** | Extended manifest; static registration; in-process invoke |
| **P2** | HostServices DI; JSON Schema config validation |
| **P3** | WASM sandbox + WIT host imports |
| **P4** | Dynamic WASM load from CAS; local discovery |
| **P5** | Signing, trust tiers, workspace policy |
| **P6** | Remote registry; conformance marketplace |
| **P7** | Native worker sandbox (optional T2) |

---

## 18. Open Decisions

1. **WIT vs custom C ABI** as primary cross-language plugin interface
2. **abi_stable** vs minimal C vtable for native dynamic plugins
3. **Plugin bundle format** — tar.zst vs single WASM component
4. **Remote registry** — self-hosted vs central `registry.s4mp.dev`
5. **MetricProvider** merge back into Analyzer as optional sub-trait?

---

## 19. Summary

The S4MP plugin system:

1. **Traits only at the boundary** — seven role traits + base `Plugin`
2. **Artifact I/O** — core never sees parser or LLM internals
3. **Static registration first**, WASM dynamic loading for ecosystem
4. **Security** — signing, trust tiers, capability permissions, sandbox by default for third-party
5. **Version compatibility** — API semver + lockfile artifact pins
6. **Discovery** — built-in → workspace → local → remote
7. **Configuration** — layered, schema-validated, secrets via host
8. **Dependency injection** — `HostServices` + `PluginRegistry` + factory pattern; no globals

The core orchestrates. Plugins implement. The store remembers. Nothing else.
