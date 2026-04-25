# Coding Standards

Baseline engineering standards for all projects. Project-specific conventions in `CLAUDE.md` (or equivalent) **always override** anything here on conflict. This document encodes defaults, not laws.

Audience: Claude Code and human contributors. Treat every rule below as the assumed baseline unless explicitly relaxed by the project.

---

## 1. Core Principles

| Principle | Operational meaning |
|---|---|
| **Correctness over cleverness** | Boring, obvious code beats clever code. If a reviewer needs context to follow it, rewrite it. |
| **Explicit over implicit** | Type annotations, named arguments where supported, explicit imports, explicit error variants. |
| **Composability over inheritance** | Small modules with clear contracts. Inheritance is reserved for narrow cases (UI primitives, error hierarchies). |
| **Determinism over magic** | No reflection-driven autowiring, no monkey-patching, no globals mutated at import time. |
| **Failure-first design** | Surface failure modes in types and signatures. The happy path is the easy path; the unhappy path must also be obvious. |
| **Local reasoning** | A reader should understand a function from its signature, body, and direct dependencies — not from cross-file side effects. |
| **Boring tooling** | Use the ecosystem's default toolchain unless there is a measured reason otherwise. |
| **Reversibility** | Prefer changes that are easy to undo. Schemas, public APIs, and dependency choices compound — treat them as architectural decisions. |

When these principles conflict with each other, pick the option that minimizes the cognitive load on the next person to read this code six months from now.

---

## 2. Language Standards

### 2.1 TypeScript

All TypeScript code targets `"strict": true` plus the additional strictness flags below. These are non-negotiable defaults.

```jsonc
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noImplicitOverride": true,
    "noFallthroughCasesInSwitch": true,
    "forceConsistentCasingInFileNames": true,
    "isolatedModules": true,
    "verbatimModuleSyntax": true,
    "esModuleInterop": true,
    "skipLibCheck": true
  }
}
```

**Type rules:**

- Never use `any`. Use `unknown` and narrow. Casting to `any` to silence the compiler is a code smell that requires a comment explaining why.
- Never use non-null assertions (`!`) to silence the compiler. Refactor to make nullability explicit, or narrow with a guard.
- Prefer `interface` for object shapes; prefer `type` for unions, intersections, mapped types, and conditional types.
- Use `readonly` on properties and `ReadonlyArray<T>` (or `readonly T[]`) on arrays whenever mutation is not required by the API contract.
- Use `satisfies` for type-safe object literals when literal type inference matters.
- Discriminated unions over boolean flags. Switch on the discriminant exhaustively and assert `never` in the `default` branch:

  ```typescript
  function assertNever(x: never): never {
    throw new Error(`Unexpected variant: ${JSON.stringify(x)}`);
  }
  ```

- Treat enums as deprecated. Use `as const` object maps and derive the union type:

  ```typescript
  export const ORDER_SIDE = { Buy: "buy", Sell: "sell" } as const;
  export type OrderSide = typeof ORDER_SIDE[keyof typeof ORDER_SIDE];
  ```

- Never use default exports. Named exports only — they are refactor-friendly, grep-friendly, and import-consistent.
- Prefer `import type` for type-only imports. With `verbatimModuleSyntax` enabled, this is enforced.
- Functions exposed across module boundaries must declare an explicit return type. Inference is fine for local helpers.

**Runtime safety:**

- Validate all data crossing a trust boundary (network, IPC, filesystem, user input) with Zod or an equivalent schema validator. Inferred TypeScript types from schemas (`z.infer<typeof Schema>`) are the single source of truth.
- Never trust `JSON.parse` output. Pipe it through a schema.
- Never trust `process.env`. Validate and coerce to a typed config object once at startup.

### 2.2 Rust

```toml
# Workspace-level Cargo.toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "warn"
panic = "deny"
todo = "warn"
```

**Type rules:**

- `#![forbid(unsafe_code)]` at every crate root unless the crate's purpose is FFI or a hot-path optimization. If unsafe is required, isolate it in a single module with a documented invariant block.
- No `panic!`, `unwrap`, or `expect` in library code or hot paths. Return `Result<T, E>`.
- Use `thiserror` for library error types (typed, derived `Display`). Use `anyhow` only at application boundaries (binary crates, top-level `main`).
- Prefer `&str` over `String` in function signatures unless ownership is required. Prefer `&[T]` over `Vec<T>`.
- For numerical computation, default to `f64`. Use `f32` only when memory pressure or hardware constraints justify it.
- Newtype pattern for domain primitives: `struct Symbol(String)`, `struct Basis(f64)`. Prevents accidental misuse and gives a place to attach validation.
- Derive `Debug` on every public type. Derive `Clone` only when needed; derive `Copy` only for small POD types.
- Use `#[non_exhaustive]` on enums and structs that may grow.

**Concurrency:**

- Use `tokio` for async I/O, `rayon` for CPU-bound data parallelism. Do not mix paradigms within a single hot path.
- Prefer message passing (`tokio::sync::mpsc`, `crossbeam::channel`) over shared mutable state.
- When shared state is required, prefer `Arc<RwLock<T>>` for read-heavy workloads, `Arc<Mutex<T>>` otherwise. Document the lock ordering for any module that holds more than one lock.

### 2.3 Python (when applicable)

- Python 3.12+ baseline. Type hints on every public function. `mypy --strict` or `pyright --strict` clean.
- Use `pydantic` v2 for runtime schema validation. Treat as the Zod equivalent.
- Use `uv` for dependency and environment management. `requirements.txt` is for export only.
- Format with `ruff format`. Lint with `ruff check`. No `black` + `flake8` + `isort` chain — `ruff` replaces all three.
- No `from x import *`. No mutable default arguments. No `print` for diagnostics — use `logging`.

---

## 3. Naming Conventions

| Element | Convention | Example |
|---|---|---|
| TypeScript files (modules) | `kebab-case.ts` | `portfolio-snapshot.ts` |
| TypeScript files (React components) | `PascalCase.tsx` (or `kebab-case.tsx` if project mandates) | `EquityCurve.tsx` |
| Variables, functions | `camelCase` | `computeReturns` |
| Types, interfaces, classes | `PascalCase` | `BacktestResult` |
| True constants | `SCREAMING_SNAKE_CASE` | `MAX_ITERATIONS` |
| Derived/computed top-level values | `camelCase` | `defaultConfig` |
| Zod schemas | `PascalCaseSchema` | `BacktestConfigSchema` |
| Database tables, columns | `snake_case` | `portfolio_snapshots`, `cost_basis` |
| Rust modules, functions | `snake_case` | `compute_returns` |
| Rust types, traits | `PascalCase` | `PriceFeed` |
| Rust constants | `SCREAMING_SNAKE_CASE` | `DEFAULT_BENCHMARK` |
| Environment variables | `SCREAMING_SNAKE_CASE` with project prefix | `CHRDFIN_TIINGO_API_KEY` |

**Semantics:**

- Avoid abbreviations except universally recognized ones (`id`, `url`, `db`, `http`, `api`). Prefer `request` over `req`, `response` over `res`, `index` over `idx` (except in tight loops where it is idiomatic).
- Boolean variables and properties read as predicates: `isLoading`, `hasErrors`, `canSubmit`, `shouldRetry`. Avoid `flag`, `status` (unless it's a string union).
- Functions are verb phrases: `computeReturns`, `validatePayload`, `loadConfig`. Pure functions returning values lean on the noun: `returns(...)`, `mean(...)` is acceptable for math primitives.
- Event handlers prefix with `handle` (within a component) or `on` (in props): `handleSubmit`, `onSelect`.
- Avoid Hungarian notation (`strName`, `iCount`) and type suffixes (`fooObject`, `barClass`).

---

## 4. Type Safety and Schema Validation

- Schemas are the single source of truth for shapes that cross trust boundaries. Never define a TypeScript interface and a Zod schema for the same shape; derive the type from the schema.
- Pair "form schemas" (UI-friendly: percentages 0-100, formatted dates) with "wire schemas" (backend-aligned: fractions 0-1, ISO dates) when the representations differ. Convert at the boundary, not inside business logic.
- Never `safeParse` and ignore the failure. Always handle `result.success === false` explicitly: render an error, throw a typed error, or log and reject.
- Centralize schemas in a shared package or module. Forms, API clients, and IPC consumers all import from the same source.
- Mirror server-side validation client-side, but never trust client-side validation alone. The wire schema is enforced on both sides.

---

## 5. Error Handling

### 5.1 General

- Errors are values. Treat them with the same rigor as success values.
- Distinguish recoverable errors (return `Result<T, E>` or equivalent) from programmer errors (panic, throw — these crash the process or surface as bugs).
- Never swallow errors silently. At minimum, log with structured context. Empty `catch` blocks are a code smell that requires a comment.
- Never throw or return raw strings. Use typed error classes/enums with stable discriminants.
- Error messages are written for humans. Include the action that failed and the relevant context (id, file, parameter), but never include secrets, full file contents, or PII.

### 5.2 TypeScript

- Use a `Result<T, E>` discriminated union for expected failures in pure logic:

  ```typescript
  export type Result<T, E = Error> =
    | { readonly ok: true; readonly value: T }
    | { readonly ok: false; readonly error: E };
  ```

- Use exceptions for unexpected failures (network unavailable, out-of-memory). Catch them at the outermost boundary that can render or log meaningfully.
- Custom error classes extend `Error`, set `name`, and tag with a discriminant:

  ```typescript
  export class ValidationError extends Error {
    readonly kind = "ValidationError" as const;
    constructor(message: string, readonly issues: readonly ZodIssue[]) {
      super(message);
      this.name = "ValidationError";
    }
  }
  ```

### 5.3 Rust

- Library errors: `#[derive(thiserror::Error, Debug)]` with variants per failure mode. Implement `From` for upstream errors via `#[from]`.
- Application errors: `anyhow::Result` is acceptable in `main` and CLI entry points.
- Never `.unwrap()` in non-test code. `.expect("explanation")` is acceptable only when the invariant is enforced elsewhere — and then only with a message describing the invariant.

### 5.4 Retry policy

- Retry policy lives at the layer closest to the failure (the HTTP client, the database driver), not at the consumer.
- Never retry on validation errors, authentication failures, or 4xx responses (except 408, 429).
- Use exponential backoff with jitter for retries on transient failures. Cap total retry time.
- Idempotency tokens for any retry-eligible mutation against external systems.

---

## 6. Async and Concurrency

- Async functions in TypeScript: never mix `.then()` chains with `await`. Pick one per function — `await` for almost everything.
- Always handle promise rejections. Top-level scripts and event handlers must wrap async calls in try/catch or `.catch()`. Unhandled rejection crashes are bugs.
- Never fire-and-forget without explicit intent. If a promise is intentionally not awaited, prefix with `void`:

  ```typescript
  void backgroundSync(); // explicit fire-and-forget
  ```

- Parallelize independent work with `Promise.all` (fail-fast) or `Promise.allSettled` (collect all outcomes). Avoid sequential awaits when calls do not depend on each other.
- In Rust, `async fn` returns a `Future` that does nothing until polled. Never construct a future and drop it.
- Cancel-safety: tokio tasks must be cancel-safe at every `.await` point that may be aborted. Document cancel-safety guarantees on public async APIs.

---

## 7. Testing

### 7.1 Coverage targets

- Unit tests for all pure logic and computation. 100% coverage of branches in financial/statistical code is the goal, not a nice-to-have.
- Integration tests for IPC, database, and external API adapters. Mock the network at the HTTP-client layer, not at the business-logic layer.
- Component tests for non-trivial UI logic (forms, conditional rendering, async flows). Skip snapshot tests except for stable design-system primitives.
- E2E tests for critical user flows only. They are slow and brittle; treat them as a smoke test, not a coverage tool.

### 7.2 Numerical testing

- Floating-point assertions must specify tolerance. Default: 1e-9 for math primitives, 1e-4 for compounded financial calculations, 1e-2 for Monte Carlo summary statistics.
- Use `approx` (Rust) or `expect.toBeCloseTo` (Vitest) with explicit precision. Never assert `===` on `f64`/`number`.
- Reference values come from a trusted source (NumPy, R, published example) and are commented with the source.

### 7.3 Test structure

- File location: colocated with source. `foo.ts` -> `foo.test.ts`. `foo.rs` -> `#[cfg(test)] mod tests` inline.
- Test names read as sentences: `it("rejects weights that don't sum to 1.0", ...)`.
- Arrange-Act-Assert structure. Empty lines separate the three phases for readability.
- One assertion per test where possible. When asserting multiple properties of one object, group with a descriptive label (`describe`).
- Mock at the boundary, not internally. If a function takes a `PriceFeed` trait, pass a fake `PriceFeed`; do not patch global imports.
- No tests that depend on wall-clock time. Inject a clock or use a fake timer.
- No tests that depend on network. Mock or skip.

### 7.4 Property tests

For algorithms with non-trivial invariants (sorting, allocation, accounting), write property tests with `fast-check` (TS) or `proptest` (Rust). Examples: "rebalanced weights always sum to 1.0", "round-trip serialize/deserialize is identity".

---

## 8. Documentation and Comments

- Every public function, type, and module exported from a package has a doc comment. JSDoc for TypeScript, rustdoc for Rust, docstrings for Python.
- Doc comments describe what, why, and contract. They do not restate the signature. The reader already sees the signature; the doc adds context.
- Prefer `// because X` comments over `// what` comments. The code says what; comments say why.
- Document non-obvious tradeoffs inline. If a comment starts with "This is a hack because..." or "We do this instead of the obvious approach because...", it earns its place.
- Never leave commented-out code in commits. Use git history.
- TODO comments include an owner and a tracking link or issue number: `// TODO(chris, #142): replace with streaming parser`.

**Documentation files:**

- `README.md` covers: what the project is, how to install, how to run, where to find more.
- `CLAUDE.md` covers: project conventions, Claude Code instructions, hard rules.
- Architectural decisions go in `docs/` as ADRs (Architecture Decision Records) for any decision that's expensive to reverse.

---

## 9. Code Organization

- One concept per file. A file that exports more than one significant abstraction is a candidate for splitting.
- Public surface explicit. Use barrel exports (`index.ts`) sparingly — they help consumers but hurt tree-shaking and tooling. Re-export only what should be public.
- Domain boundaries are real. Cross-domain imports go through a shared types package or an explicit interface — never through "reach into another domain's internals".
- Side-effectful imports are forbidden. A module that, when imported, mutates global state is a bug waiting to happen. Initialize explicitly in entry points.
- Circular imports are forbidden. They indicate a missing abstraction.
- Group imports in this order, separated by blank lines:
  1. Standard library / framework (e.g. React, Node built-ins)
  2. External packages (alphabetized)
  3. Internal monorepo packages (alphabetized)
  4. Relative imports (deepest first)
- Configure ESLint's `import/order` and `import-x/order` to enforce this automatically.

---

## 10. Dependency Management

- Pin exact versions in lockfiles (`pnpm-lock.yaml`, `Cargo.lock`, `uv.lock`). Commit lockfiles for both libraries and applications — including for Rust libraries, despite the older guidance, for build reproducibility.
- Use `pnpm` for Node.js workspaces. Use the latest stable major.
- Audit dependencies before adding. The bar for a new dependency:
  - Is it actively maintained (commits in the last 6 months)?
  - Does it have a permissive license (MIT, Apache-2.0, BSD)?
  - What is its transitive footprint?
  - Is there a meaningfully smaller alternative?
- License compatibility is enforced. GPL/AGPL dependencies are forbidden in code intended for distribution unless explicitly approved.
- Update dependencies on a cadence (monthly minimum), not reactively. Use Renovate or Dependabot. Read changelogs before merging majors.
- Production code never depends on a `*` or `latest` version specifier.

---

## 11. Git and Version Control

### 11.1 Commits

- Conventional Commits format:

  ```
  <type>(<scope>): <subject>

  <body>

  <footer>
  ```

- Types: `feat`, `fix`, `chore`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `revert`.
- Subject: imperative mood, no trailing period, ≤72 chars. "add transaction form" not "added transaction form".
- One logical change per commit. If you find yourself writing "and" in the subject, split it.
- Body explains why, not what. The diff shows what.
- Footer for breaking changes (`BREAKING CHANGE: ...`) and issue refs (`Closes #142`).

### 11.2 Branches

- `main` is always deployable. Merges to `main` require passing CI.
- Feature branches: `feat/<short-description>`, `fix/<short-description>`. Keep them short-lived (≤1 week ideally).
- Rebase before merging. Squash merges for feature branches; preserve history for long-lived integration branches.

### 11.3 Pull requests

- PR description states: what changed, why, how it was tested, any tradeoffs or follow-ups.
- PRs are small. ≤400 lines of diff is the target; ≥1000 lines requires justification.
- Self-review before requesting review. Read your own diff. Add inline comments on non-obvious decisions.

---

## 12. Security Baseline

- Never commit secrets. `.env` is gitignored; only `.env.example` is tracked.
- Validate `process.env` once at startup with a schema. Fail fast on missing required keys.
- Secrets in production come from a dedicated secret store: OS keychain (desktop), environment variables (server), or a secrets manager (cloud). Never plaintext config files.
- All external inputs validated at the trust boundary. SQL via parameterized queries — string interpolation is a bug, not a style preference.
- Hashing: `argon2id` for passwords. `sha-256` is fine for content addressing; never for passwords.
- HTTPS only for external calls. Pin certificates only when threat-model justifies it.
- Cross-site: CSP headers, `SameSite=Strict` cookies for auth, no `dangerouslySetInnerHTML` without sanitization.
- Dependency vulnerability scans run in CI (`pnpm audit`, `cargo audit`). High/critical advisories block merges.
- Logs never contain: passwords, tokens, full credit card numbers, full SSNs, raw user PII. Mask at the logging layer.

---

## 13. Performance

- Measure before optimizing. Benchmark with `criterion` (Rust), `tinybench` (TS), or `pytest-benchmark` (Python). Production performance matters more than micro-benchmarks.
- Algorithmic complexity beats constant-factor optimization. Reach for an O(n) solution before tuning O(n²).
- For hot paths in TypeScript: avoid allocations in tight loops, prefer typed arrays for numerical work, batch DOM mutations.
- For hot paths in Rust: prefer iterator chains over manual loops (the optimizer handles them well), prefer `&[T]` over `Vec<T>` clones, profile with `cargo flamegraph` before guessing.
- Memoize at the query/cache layer (TanStack Query, Redis), not in business logic. Cache invalidation is hard; centralize it.
- Lazy-load route bundles. Code-split at route boundaries by default. Eagerly load only what's required for first paint.
- Database queries: indexes for every column used in `WHERE`, `JOIN`, or `ORDER BY`. Read query plans (`EXPLAIN`) for any non-trivial query.

---

## 14. Logging and Observability

- Structured logging only. Levels: `error`, `warn`, `info`, `debug`, `trace`.
- Use `tracing` (Rust), `pino` (Node), `structlog` (Python). Logs are JSON in production; pretty-printed in development.
- Every log line includes: timestamp, level, message, and structured fields (request id, user id where applicable). No string interpolation of structured data — pass as fields.
- `info` for state transitions and significant events. `debug` for verbose troubleshooting. `warn` for recoverable anomalies. `error` for failures requiring attention.
- Spans/traces around every IPC boundary, every external call, every long-running computation.
- Metrics for: request rate, error rate, latency (p50/p95/p99), and any domain-specific counters. Expose via Prometheus or equivalent.
- No `console.log` left in production code. Use the project logger.

---

## 15. Code Review Heuristics

When reviewing (or self-reviewing), check in this order:

1. **Correctness**: does it solve the stated problem? Are edge cases handled?
2. **Tests**: are there tests? Do they cover the new behavior, including failure modes?
3. **Types**: are types tight? Any `any`, `unknown` without narrowing, non-null assertions?
4. **Error paths**: what happens when each external call fails? Is failure surfaced?
5. **Naming**: does each name describe what it is, in the language of the domain?
6. **Scope**: is the change focused? Could it be split into smaller PRs?
7. **Side effects**: any new global state, mutable singletons, side-effectful imports?
8. **Dependencies**: any new dependencies? Are they justified?
9. **Performance**: any obvious hot-path issues — N+1 queries, unbounded loops, large allocations?
10. **Security**: any new trust boundary? Any input not validated? Any secret risk?

The reviewer's job is to ask questions, not to rewrite the code. If you find yourself wanting to rewrite a section, leave a specific comment with the suggestion and the reasoning.

---

## 16. Refactoring Discipline

- Refactor in commits separate from feature commits. A PR titled `feat: add backtest export` should not also rename twelve unrelated files.
- Tests pass before, during, and after. If a refactor breaks tests, the refactor is wrong or the tests are wrong — fix one before continuing.
- Mechanical refactors (rename, extract, inline) come from tooling. Manual find-and-replace introduces bugs.
- Deprecate before deleting public APIs. Mark with `@deprecated` JSDoc or `#[deprecated]` Rust attribute, document the replacement, allow at least one minor version before removal.

---

## 17. Anti-Patterns to Avoid

| Anti-pattern | Why it's bad | What to do instead |
|---|---|---|
| `any`, `as any` | Disables type checking | Use `unknown` and narrow |
| Default exports | Refactor-hostile, inconsistent imports | Named exports |
| Catch-and-swallow | Hides bugs | Log + rethrow, or handle explicitly |
| `setTimeout` for synchronization | Race-prone, wall-clock dependent | Promises, signals, or explicit state machines |
| Manual JSON validation | Drift between schema and code | Zod / pydantic / serde with validation |
| Ambient mutable state | Untestable, surprising | Pass dependencies explicitly |
| Boolean parameters | Unreadable at call site | Object parameters or separate functions |
| Long parameter lists | Easy to misorder | Object parameter with named fields |
| Deeply nested ternaries | Unreadable | Early returns or extracted functions |
| `// eslint-disable` without comment | Hides intent | Disable + comment the justification |
| `unwrap()` / non-null `!` | Panics in production | `Result` / `Option` handling, or narrow |
| Mixed sync/async in one function | Hard to reason about | Pick one paradigm per function |
| God objects, god modules | Become bottlenecks | Extract by responsibility |
| Premature abstraction | Wrong abstraction is worse than duplication | Wait for the third occurrence |
| Comments restating code | Noise | Delete; let names speak |

---

## 18. When to Deviate

Standards exist to make the common case fast and the unusual case visible. When you have a measured, articulable reason to deviate:

1. Document the deviation inline with a comment explaining why.
2. Confine the deviation to the smallest scope possible.
3. If the deviation recurs, it's a candidate for becoming the new standard — propose the change.

Standards that are routinely ignored are worse than no standards. If a rule here is consistently producing friction with no clear benefit, raise it for revision rather than silently violating it.

---

## 19. Hierarchy of Authority

When guidance conflicts, the order of precedence is:

1. **Project-specific `CLAUDE.md`** (or equivalent) at the repository root
2. **Project-specific docs** in `docs/` referenced by `CLAUDE.md`
3. **This file** (`.claude/instructions/coding-standards.md`)
4. **Ecosystem defaults** (TypeScript handbook, Rust API guidelines, etc.)

If this file conflicts with a project's `CLAUDE.md`, the `CLAUDE.md` wins. If unsure, ask before proceeding.
