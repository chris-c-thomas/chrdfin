# Architecture Principles

System-design baseline. Defines how modules, services, and data flow are structured — the "why this lives there" and "why this talks to that." Companion to `coding-standards.md` (what good code looks like in the small) and `code-review.md` (how reviews run).

Audience: Claude Code and human contributors making structural decisions. The goal is to keep systems comprehensible and changeable as they grow. Architecture decisions compound; this document encodes the defaults so the compounding works in your favor.

---

## 1. Core Principles

| Principle | Operational meaning |
|---|---|
| **Boundaries are real** | Module / package / service boundaries exist to hide implementation. Imports across them go through a defined contract, not a reach-in. |
| **Contracts are explicit** | Every boundary has a typed, validated contract. Schemas at trust boundaries; types at internal boundaries. |
| **Dependencies point inward** | Stable code does not depend on volatile code. Domain does not depend on infrastructure. |
| **Failure is in the type system** | Functions that can fail return a result that says so. The compiler enforces handling. |
| **Single source of truth** | Every fact lives in one place. Derived state is computed, not stored, unless caching is explicit and invalidated correctly. |
| **Composition over orchestration** | Assemble small, focused units. A 200-line orchestrator coordinating 8 services is usually wrong; a 30-line one coordinating 3 is usually right. |
| **Reversibility** | Prefer designs that can be changed cheaply. Treat schemas, public APIs, and external integrations as expensive — get them right or design them to evolve. |
| **Observable by default** | Every boundary emits enough signal to debug from logs and metrics alone. |
| **Latency is a feature** | Async, batching, caching, and parallelism are architectural choices, not optimizations. Decide early. |
| **Security is a property of the boundary** | Trust changes at boundaries. Validation and authorization happen at boundaries. |

When principles conflict, default to the one that preserves long-term changeability. The system you ship in six months matters more than the system you ship today.

---

## 2. Module and Package Boundaries

A boundary exists to:

1. Hide implementation details so they can change freely.
2. Define a stable contract that consumers depend on.
3. Limit blast radius when something breaks.

### 2.1 What constitutes a boundary

| Boundary | Coupling tolerated | Versioning |
|---|---|---|
| Function / type within a module | Tight — one author, one change | None |
| Module within a package | Medium — shared reviewer, shared release | Internal |
| Package within a monorepo | Loose — shared types only | Workspace protocol; internal semver |
| Service / process | Wire-protocol only | Strict semver, contract tests |
| External system | Versioned API contract | Strict semver, anti-corruption layer |

Each boundary is a place where: contracts are validated, errors are translated, observability hooks fire, and types narrow.

### 2.2 Rules for boundaries

- **Cross-domain imports go through a shared types package or interface.** Domain A does not import from `domain-b/internal/*`. If domain A needs something from domain B, that thing is in a shared package or exposed via a domain-B public API.
- **No circular dependencies between modules or packages.** A circular dependency means a missing abstraction. Extract the common dependency to a third module.
- **Public surface is explicit and minimal.** Index files re-export only what consumers need. Internals are not re-exported.
- **Internal types do not leak across boundaries.** If a type is implementation-specific, it stays internal. If it's a contract, it lives in the shared types package.
- **Boundaries are enforced by tooling, not vigilance.** ESLint `import/no-restricted-paths`, Cargo's module privacy, or workspace-level lint rules. A boundary you can't see in CI doesn't exist.

### 2.3 Anti-patterns

- "God packages" that everything depends on. Symptom: changing one type rebuilds the whole tree.
- Reaching into another package's `src/internal/*` for a "quick" need. Symptom: the other package can't refactor without breaking yours.
- Re-exporting everything from `index.ts` to be "convenient." Symptom: tree-shaking fails, refactors thrash imports across the codebase.

---

## 3. Dependency Direction

Stable depends on volatile? No. Volatile depends on stable.

### 3.1 The layered model

```
[ External integrations ] ←  most volatile, may swap
        ↓
[ Adapters / Infrastructure ]
        ↓
[ Domain logic ]            ←  stable, knows nothing about infrastructure
        ↑
[ Application / orchestration ]
        ↑
[ UI / API surface ]        ←  most volatile, may swap
```

- **Domain logic** (computation, business rules, invariants) has no imports from infrastructure, UI, or external systems. It accepts inputs, returns outputs, and is fully testable in isolation.
- **Adapters** translate between domain types and external systems (HTTP clients, database drivers, file formats). One adapter per external system.
- **Application layer** orchestrates domain + adapters to produce features. It knows about both but neither knows about it.
- **UI / API surface** is the entry point. Routes, components, command handlers. They know about the application layer but the application layer never reaches up.

### 3.2 Practical consequences

- A change to a database driver affects the adapter only.
- A change to a UI framework affects the UI surface only.
- A change to a domain rule affects domain logic and any adapter that exposed the changed contract.
- Tests on domain logic do not need a database, a network, a file system, or a clock.

### 3.3 Inversion at boundaries

The domain defines an interface; the adapter implements it. The dependency arrow points from adapter to domain, not the other way around.

```typescript
// In domain (no infrastructure imports)
export interface PriceFeed {
  getPrice(symbol: string, date: ISODateString): Promise<Result<Price, PriceFeedError>>;
}

// In adapter (depends on domain)
import type { PriceFeed } from "@chrdfin/domain";
export class TiingoPriceFeed implements PriceFeed { /* ... */ }
```

The domain knows: "I need a `PriceFeed`." It does not know: "I need a Tiingo HTTP client." Swapping providers is an adapter change, not a domain change.

---

## 4. Contract-First Design

Every boundary has a contract. Define it before implementing either side.

### 4.1 Contract artifacts by boundary type

| Boundary | Contract artifact |
|---|---|
| Internal function | TypeScript / Rust signature |
| Cross-package | Shared types in a dedicated package, with schemas where validation is needed |
| IPC (Tauri command, Web Worker, etc.) | Shared schema (Zod) + matching server-side type (serde, etc.) |
| HTTP API | OpenAPI 3.1+, generated typed client, runtime validation on server |
| Event / message | Schema (Zod, Avro, Protobuf) versioned with the producer |
| Database | DDL with explicit constraints; migrations versioned in source |

### 4.2 Schema as source of truth

When a boundary requires runtime validation, the schema is the source of truth. Types are derived from the schema; documentation references the schema.

```typescript
// One source of truth
export const BacktestConfigSchema = z.object({ /* ... */ });
export type BacktestConfig = z.infer<typeof BacktestConfigSchema>;
```

Never define a schema and a hand-written type for the same shape. They will drift.

### 4.3 Form vs wire schemas

When the UI representation and the backend representation diverge (percentages 0-100 vs fractions 0-1; formatted dates vs ISO; nullable vs optional), define both schemas and an adapter:

```typescript
const FormSchema = z.object({ weightPercent: z.number().min(0).max(100) });
const WireSchema = z.object({ weight: z.number().min(0).max(1) });

function toWire(form: FormInput): WireInput {
  return { weight: form.weightPercent / 100 };
}
```

Conversion happens at the boundary, not inside business logic.

### 4.4 Contract evolution

- **Additive changes** (new optional field, new enum variant in a `#[non_exhaustive]` enum) are usually safe. Test against current consumers.
- **Removing or changing** a field is a breaking change. It requires a deprecation cycle (§13).
- **Renaming** is two changes: add the new field, deprecate the old, remove later. Never rename in-place.
- **Tightening validation** (making a field required, narrowing a range) breaks producers. Coordinate.
- **Loosening validation** (making a field optional, widening a range) breaks consumers if they relied on the constraint. Coordinate.

---

## 5. Data Flow

Data flows in one direction at a time. A function reads from its inputs and writes to its outputs. Functions that read and write the same state are an exception requiring justification.

### 5.1 Read paths

- Read paths fan out: one query may hit multiple sources (cache, primary, derived).
- Cache invalidation is centralized. Caches do not invalidate themselves; the producer of a change invalidates dependent caches.
- Reads are cheap by default. Pagination, indexes, projections, and caching are designed in — never bolted on after the fact.

### 5.2 Write paths

- Write paths fan in: one mutation goes through one path. No "two ways to update X."
- Writes are validated at the entry point and again at the persistence layer. Defense in depth.
- Writes emit events (or invalidate caches) on success. Event payloads are typed schemas.
- Writes that span multiple stores require explicit transaction or saga design. Don't fake it with try/catch.

### 5.3 Derived state

- Derived state is computed from source state. Storing it requires explicit caching strategy.
- For UI: derived state is computed on render unless profiling shows it's a hot path. Premature memoization is its own bug class.
- For analytics: materialized views or precomputed aggregates are fine; their invalidation is part of the schema design.

### 5.4 Single source of truth

Every fact has one owner. Examples:

- User's current portfolio composition: owned by the database. Cached in the query layer, not duplicated in component state.
- Theme preference: owned by user settings. Components read from a provider; they don't each maintain their own copy.
- Form state: owned by the form library (RHF) until submit; URL search params on submit if shareable; database after persistence.

Multiple "sources of truth" for the same fact will drift. Architect to prevent it.

---

## 6. State Management

State is categorized by lifetime and ownership. Different categories use different tools.

| Category | Lifetime | Tool |
|---|---|---|
| **Server / persistent** | Outlives every session | Database, query cache (TanStack Query) |
| **URL** | Survives reload, shareable | Router search params |
| **Session** | Per-app-launch | In-memory store, context provider |
| **Component-local** | Per-mount | `useState`, `useReducer` |
| **Form** | Per-form-lifetime | Form library (RHF + Zod) |

Rules:

- **Don't store derived state in component state.** Compute it. If profiling shows the cost is real, memoize.
- **Don't duplicate server state in component state.** Read from the query cache. The cache is the local view of server state.
- **Don't put session state in localStorage.** It's untyped, untyped-loss-prone, and survives across contexts where it shouldn't. Use it only when nothing else fits and validate on read.
- **Lift state to the lowest common ancestor**, but no higher. Premature lifting creates re-render fan-out.
- **Global state is suspect.** A global store is a sign that boundaries are wrong. Most "global" state is either server state, URL state, or context-scoped state.

---

## 7. Error Propagation Across Boundaries

Errors are part of the contract. They cross boundaries deliberately.

### 7.1 Within a layer

Use the layer's idiomatic mechanism: `Result<T, E>` in Rust, typed errors in TypeScript with discriminated unions, exceptions for truly unexpected failures.

### 7.2 Across a boundary

Translate. Never leak internal error types across boundaries.

```rust
// Wrong: leaks DuckDB error type to caller
pub async fn get_quote(symbol: &str) -> Result<Quote, duckdb::Error>;

// Right: typed domain error, with internal translation
#[derive(thiserror::Error, Debug)]
pub enum PriceFeedError {
    #[error("symbol not found: {0}")]
    SymbolNotFound(String),
    #[error("no data for date range")]
    NoData,
    #[error("internal error")]
    Internal(#[source] anyhow::Error),
}
```

The boundary owns the error vocabulary. Internal errors map to domain errors at the boundary; consumers see only the domain vocabulary.

### 7.3 Across IPC / network

- Backend errors translate to user-readable strings (or structured error codes) before crossing the wire. The frontend renders them; it does not parse internal error types.
- Always return a typed error variant, never a stringly-typed error message. Frontend code switches on the variant, not on substring matches.
- Status codes (HTTP) or error discriminants (IPC) are part of the contract. Adding new ones requires consumer updates.

### 7.4 Retry policy

Retry policy lives at the layer closest to the failure. The HTTP client retries transient network errors. The database driver retries transient lock errors. The application layer does not retry — by the time an error reaches it, the lower layer has done its retries and the error is final.

---

## 8. Async Boundaries

### 8.1 Where async exists

- **I/O boundaries are async.** Network, disk, IPC. The function signature reflects it.
- **Pure computation is sync.** A `computeReturns(prices)` function does not return a Promise, even if it's slow. Slow computation goes off the main thread (worker, Rust task) at the boundary, not via async signature.
- **Don't async-stain unnecessarily.** A function that calls one async helper does not need to be `async` if the result is consumed by a Promise-returning chain. But: clarity often beats minimalism here. Pick a convention and stick to it.

### 8.2 Cancellation

- Long-running async work is cancelable. AbortSignal in TypeScript; CancellationToken or `tokio::select!` with a cancellation channel in Rust.
- Cancellation is part of the contract. Document it on public async APIs.
- Cancel-safety: in Rust, every `.await` point in cancelable code is safe to drop. State is committed only after the suspending operation completes.

### 8.3 Concurrency control

- **Backpressure**: producers slow down when consumers fall behind. Bounded channels, semaphores, or rate limiters.
- **Parallelism vs concurrency**: independent CPU work uses Rayon (Rust) or Workers (browser). Concurrent I/O uses async.
- **Connection pools and rate limiters** belong at the adapter layer. Domain code asks "fetch this"; the adapter handles concurrency.

---

## 9. Trust Boundaries and Security

Trust changes at boundaries. Validation, authentication, and authorization happen at the boundary, not deeper.

### 9.1 Identifying trust boundaries

- Network ingress / egress
- IPC between processes (browser ↔ Rust, web worker ↔ main thread)
- Filesystem reads of user-controlled paths
- Process arguments and environment variables
- Anywhere user-supplied data enters domain logic

### 9.2 At every trust boundary

- **Validate input** with a schema. Reject malformed input with a typed error before it touches business logic.
- **Authenticate** if applicable. Identity is established at the boundary; downstream code trusts it.
- **Authorize** if applicable. The check is at the boundary or just inside; not buried four functions deep.
- **Sanitize for the next boundary** if data continues onward. SQL → parameterized queries. HTML → escaping. Shell → no string concatenation.
- **Rate limit / quota check** for external-facing boundaries.
- **Log enough to investigate**. Source IP, user id, action attempted, outcome.

### 9.3 Secrets

- Secrets live in a secret store: OS keychain (desktop), environment (server), secrets manager (cloud). Never plaintext config in version control.
- Secrets are loaded once at startup, validated against a schema, and injected. They do not float as ambient globals.
- Secrets do not appear in logs, error messages, URLs, or stack traces.

### 9.4 Defense in depth

A single security layer is a single point of failure. Validation at the API boundary AND at the persistence boundary. Authorization at the route AND at the data access. Each layer assumes the previous one failed.

---

## 10. Observability

A system without observability is a system without operability. Observability is designed in, not added later.

### 10.1 The three pillars

| Signal | Purpose | Tool |
|---|---|---|
| **Logs** | What happened | Structured logger (`tracing`, `pino`) |
| **Metrics** | How much, how fast | Prometheus, OTel metrics |
| **Traces** | Where time was spent | OTel traces |

### 10.2 Defaults

- Every boundary emits a span. Every external call is a child span.
- Every error is logged once at the layer that translates it. Don't log-and-rethrow at every layer — pick one.
- Every long-running operation emits progress events.
- Every public function's invocation can be enabled at `debug` level without code changes.

### 10.3 Cardinality discipline

Metrics labels and trace attributes have bounded cardinality. User id is fine; arbitrary user input is not. High-cardinality labels destroy metrics backends.

### 10.4 What to never log

Passwords, API keys, full credit card numbers, full SSNs, raw user PII (names, emails) without redaction, full request bodies that may contain any of the above.

---

## 11. Configuration

Configuration is data; it deserves the same rigor as code.

- **Validated at startup.** A schema parses environment variables, config files, or whatever source. Failure is fatal — fail fast, log clearly.
- **Typed thereafter.** No `process.env.X` reads scattered through the codebase. One `config` object, fully typed, passed where needed.
- **Sourced by environment.** Defaults for dev. Overrides via env vars for staging/production. Secrets from the secret store.
- **No code paths gated on config that isn't typed.** A `if (process.env.FEATURE_X === "true")` scattered in business logic is a bug.

Feature flags follow the same rules: typed at startup, evaluated through a typed API, never as ambient string lookups.

---

## 12. Versioning and Evolution

Public surfaces evolve under semver. Internal surfaces evolve freely.

### 12.1 What's public

- Anything exported from a package's index.
- Anything that crosses a process boundary (HTTP API, IPC command, event payload).
- Anything persisted (database schemas, file formats).

### 12.2 Semver in practice

| Change | Bump |
|---|---|
| Add an optional field, add a new function | Minor |
| Add an enum variant to a `#[non_exhaustive]` enum | Minor |
| Make a field required, change a type, remove a function | Major |
| Bug fix that doesn't change contract | Patch |
| Performance fix that doesn't change contract | Patch |

### 12.3 Deprecation cycle

1. Mark the API as deprecated (`@deprecated` JSDoc, `#[deprecated]` Rust attribute).
2. Document the replacement in the deprecation notice.
3. Update internal consumers to the replacement.
4. Allow at least one minor version with the deprecation in place.
5. Remove in the next major version.

Never rename in-place. Never remove without deprecation. The cost of deprecation is small; the cost of breaking consumers is large.

### 12.4 Database evolution

- Migrations are versioned and committed. They roll forward and (where feasible) roll back.
- Schema changes are additive in production: add column, backfill, switch reads, deprecate old, remove later. Never alter-then-pray.
- Destructive changes require an explicit, reviewed plan.

---

## 13. Testing Strategy by Layer

Each layer has a different testing approach. Don't test domain logic with E2E and don't test E2E with unit tests.

| Layer | Test type | What to assert |
|---|---|---|
| Domain logic | Unit (fast, no I/O) | Invariants, edge cases, numerical correctness |
| Adapters | Integration (real or close-to-real I/O) | Translates correctly, handles failures, cancels safely |
| Application | Integration with mocked adapters | Orchestrates correctly, propagates errors |
| UI | Component (React Testing Library) | User-visible behavior, accessibility |
| End-to-end | Browser automation | Critical user flows only — smoke test, not coverage |
| Contract | Schema tests + compatibility checks | Public APIs don't break consumers |

The pyramid: many unit tests, fewer integration tests, very few E2E tests. Inversions of this ratio (lots of E2E, few unit) are slow and brittle.

---

## 14. Performance Architecture

Performance is designed in. Optimizing later is bounded by the architecture you've chosen.

### 14.1 Latency budget

Establish a budget per layer. Example for a typical web request: <50ms server compute, <100ms database, <200ms total to first byte. UI: <100ms to first interaction.

Architectures that ignore the budget produce systems that get slower over time.

### 14.2 Hot path discipline

- Identify hot paths. Profile, don't guess. Common ones: request-critical paths, render loops, data ingestion pipelines.
- On hot paths: minimize allocations, batch I/O, avoid unnecessary work, prefer streaming over buffering.
- Hot paths get benchmarks in CI. Regressions block merges.

### 14.3 Caching

Caching is correctness-sensitive. Get the invalidation right or don't cache.

- **Read-through cache**: cache populated on miss. Simple, easy to reason about.
- **Write-through cache**: cache and source updated together. Stronger consistency, more code.
- **Cache-aside**: application explicitly reads and writes. Most flexible, most error-prone.

Whichever pattern: every cache has an explicit invalidation strategy and a documented consistency model. "Eventually consistent" is acceptable; "we don't know when this updates" is not.

### 14.4 Concurrency for performance

- Parallel I/O: `Promise.all` / `tokio::join!`. Independent calls in parallel, dependent calls sequential.
- Parallel compute: Rayon, Web Workers. Boundary at the adapter / application layer; domain stays single-threaded.
- Backpressure: bounded channels, semaphores. Unbounded queues are bombs.

---

## 15. Failure Modes and Resilience

Plan for failure. Every external dependency will fail eventually.

### 15.1 Failure taxonomy

| Failure | Detection | Response |
|---|---|---|
| Transient (network blip, lock contention) | Retry succeeds | Bounded retry with backoff |
| Persistent external (provider outage) | Retries exhausted | Fail the request, surface error, fall back if available |
| Logical (bad input, broken invariant) | Validation, assertion | Reject input or panic; do not retry |
| Resource exhaustion (OOM, disk full) | OS / runtime signals | Fail fast, alert |
| Dependency mismatch (schema drift) | Schema validation fails | Fail fast at startup, do not start |

### 15.2 Resilience patterns

- **Timeouts**: every external call has a timeout. No timeout = potential hang.
- **Retries**: bounded, with exponential backoff and jitter. Idempotency required for retried mutations.
- **Circuit breakers**: when a dependency is failing repeatedly, stop calling it for a window. Better to fail fast than queue up.
- **Bulkheads**: isolate concurrent work so one slow dependency doesn't drain the pool for everything.
- **Graceful degradation**: identify which features can degrade (cached data, reduced fidelity, read-only mode) when a dependency is down. Design for it explicitly.

### 15.3 What to never do

- Catch and ignore. Failures must surface or be deliberately suppressed with a comment.
- Retry forever. Bound everything.
- Hold a lock across a network call. Deadlock waiting to happen.
- Couple all features to one dependency without a fallback. Single point of failure.

---

## 16. Domain Modeling

Model the domain in domain language. The code reads like the spec, not like the database.

### 16.1 Domain primitives

Use newtypes (Rust) or branded types (TypeScript) to prevent mixing semantically-distinct values that share a primitive representation.

```typescript
type Symbol = string & { readonly __brand: "Symbol" };
type ISIN = string & { readonly __brand: "ISIN" };
// You can't accidentally pass an ISIN where a Symbol is expected.
```

```rust
pub struct Symbol(String);
pub struct Isin(String);
```

This catches bugs at compile time and creates a place to attach validation.

### 16.2 Make illegal states unrepresentable

If a field is required, type it as required. If two fields go together (e.g., `start` and `end` always present together) put them in a struct. If a value can be one of three states, use a discriminated union, not three booleans.

### 16.3 Persistence is one representation

The database schema is one representation of the domain, not THE domain. Domain types may differ from row shapes. The mapping is in the adapter layer.

For computational systems with strong invariants (financial calculations, stateful protocols), the domain types should enforce the invariants; the database is a serialization format.

---

## 17. AI / Agent System Patterns

For systems that incorporate LLMs or agentic components.

### 17.1 Boundaries

- **Model calls are external dependencies.** Wrap them in an adapter. The domain does not import OpenAI or Anthropic SDKs directly.
- **Prompts are versioned source artifacts.** They live in source control, are reviewed, and have tests.
- **Tool / function-call schemas are typed.** The same schema validates inputs and is the source of types for the executor.

### 17.2 Trust

- **Model output is untrusted input.** Treat anything coming back from a model the same way you treat raw HTTP request bodies: validate against a schema, reject malformed responses.
- **User-controlled text injected into prompts is a vector.** Prompt injection is to LLM systems what SQL injection is to databases. Sanitize, isolate, or design around it.
- **Tool execution from model output requires explicit gating.** A model "asking" to run a destructive operation does not authorize it. Authorization is a separate decision.

### 17.3 Determinism and testing

- Tests use deterministic mocks for model calls. Live model calls live in a separate eval suite that is allowed to be flaky and slow.
- Eval suites measure quality on a frozen set of cases. Track scores over time. A regression is a bug.
- Token usage and cost are observability signals. Log per-call cost and aggregate by feature.

### 17.4 Failure modes

- Model returns malformed output → schema validation fails → retry once with a "fix this" reprompt, or fail.
- Model returns refusal → distinguish from error, surface differently.
- Model is slow → timeout, fall back if possible.
- Model is unavailable → graceful degradation path defined per feature.

### 17.5 Retrieval

- Retrieval is a separate component with its own contract (input: query, output: ranked chunks).
- Retrieval evaluation is independent of generation evaluation. Bad retrieval cannot be fixed by better prompts.
- Index management (chunking strategy, embedding model, refresh cadence) is documented and versioned. Changing any of them requires reindexing and is a contract change.

---

## 18. When to Add Complexity

Most architectural complexity is added prematurely. The defaults:

- **Start with a monolith** unless you have a specific reason for service boundaries.
- **Start with synchronous calls** unless you've measured that async helps.
- **Start with one database** unless you've identified a use case the database can't serve.
- **Start with one deployment** unless you have an isolation reason.
- **Start with the simplest cache (none)** unless you've measured the read load.

Complexity earns its place by solving a measured problem, not a hypothetical one. Every layer of indirection has a cost; the cost is paid forever.

When evaluating whether to add complexity, ask:

1. What problem does this solve?
2. Have we measured the problem, or are we anticipating it?
3. What's the simpler alternative we'd have to defer?
4. What does this cost in the long term — maintenance, mental load, debugging?
5. Is the reversibility of this decision proportional to our confidence?

If the answers don't line up, defer the complexity.

---

## 19. Common Architectural Anti-Patterns

| Anti-pattern | Symptom | Resolution |
|---|---|---|
| God module / package | Touching it rebuilds everything | Split by responsibility |
| Shotgun surgery | One change requires edits in many files | Wrong abstraction; consolidate |
| Feature envy | Module A constantly reaches into B | Move logic to where the data lives |
| Anemic domain | Domain types are dumb structs; logic is in services | Push behavior onto types |
| Service-orientation by default | "Microservices" with shared database and synchronous chains | Start as a modular monolith |
| Premature abstraction | Generic framework with one implementation | Inline; abstract when the second case appears |
| Big-bang rewrite | Plan to throw out everything | Strangler pattern; migrate incrementally |
| Distributed monolith | Multiple services, every change touches all | Wrong boundaries; consider re-merging |
| Database as integration | Two services share a database for "communication" | Define a contract; one service owns the data |
| Configuration as logic | Dozens of feature flags gating fundamental behavior | Some of those flags are real branches; promote them |
| Cache without invalidation | "We'll figure it out when it becomes stale" | Caches are correctness-sensitive; design invalidation up-front |
| Snowflake environment | Production differs from dev in undocumented ways | Containerize; enforce parity |
| Untyped boundaries | "We'll just JSON it" | Schema at every boundary, even internal |
| Logging instead of metrics | Grepping for performance | Metrics are aggregates; logs are events. Use both correctly. |

---

## 20. Decision Records

Architectural decisions are durable. They get documented.

For any decision that's:

- Expensive to reverse (database choice, framework choice, language choice, vendor lock-in)
- Affects multiple teams or modules
- Establishes a pattern future code will follow
- Trades off two reasonable alternatives non-obviously

…write an ADR (Architecture Decision Record). Format:

```markdown
# ADR-NNN: Title

**Status:** [Proposed | Accepted | Superseded by ADR-MMM]
**Date:** YYYY-MM-DD

## Context
The forces at play. What problem are we solving? What constraints exist?

## Decision
What we chose and why.

## Alternatives considered
What else was on the table, and why it was rejected.

## Consequences
What this enables, what this forecloses, what we'll need to revisit.
```

ADRs live in `docs/adr/` and are numbered sequentially. They are not edited after acceptance — they are superseded by new ADRs. This creates a durable record of why the architecture is what it is.

---

## 21. Hierarchy of Authority

When guidance conflicts:

1. **Project-specific `CLAUDE.md`** at the repository root.
2. **Project-specific docs** in `docs/`, including ADRs.
3. **`coding-standards.md`** (small-scale code standards).
4. **`code-review.md`** (review process).
5. **This file** (architectural baseline).
6. **Ecosystem norms.**

Architectural rules are stickier than coding rules. Deviating from a coding rule is usually local; deviating from an architectural rule has system-wide consequences. When in doubt, write an ADR documenting the deviation rather than silently violating the rule.
