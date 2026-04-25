# Code Review

Operational guide for reviewing pull requests and self-reviewing before submitting them. Companion to `coding-standards.md` (which defines what "good code" means) and `architecture-principles.md` (which defines what "good design" means). This document defines the review process itself.

Audience: Claude Code and human reviewers. The goal of a review is to ship correct, maintainable code with the smallest reasonable cycle time. Reviews that block on style nits or relitigate decided architecture fail that goal.

---

## 1. The Job of a Reviewer

A reviewer is responsible for catching:

1. **Correctness defects** — bugs, missed edge cases, broken invariants.
2. **Maintainability hazards** — code that future readers will misunderstand or struggle to change.
3. **Security and data-integrity risks** — anything that could leak, corrupt, or expose data.
4. **Architectural drift** — changes that violate boundaries or contracts established elsewhere.

A reviewer is **not** responsible for:

- Re-running the linter or formatter. Tooling enforces style.
- Reaching personal stylistic preferences. If two approaches are both reasonable, the author's choice stands.
- Rewriting the PR. Suggestions, not rewrites. If the diff needs a rewrite, that's a conversation, not a review comment.
- Catching every typo. Spell-checkers exist; reviewers focus on substance.

The reviewer is the second pair of eyes — not the QA team, not the architect-of-record, not the gatekeeper. Approval signals "I read this and I'd be comfortable owning it." Nothing more.

---

## 2. Review Depth Tiers

Not every PR deserves the same scrutiny. Calibrate depth before reading.

| Tier | Examples | Review depth |
|---|---|---|
| **L1 — Skim** | Doc-only changes, dependency bumps under a major, generated code, formatting-only PRs, comment fixes, test name changes | Read the description. Verify CI is green. Approve unless something jumps out. |
| **L2 — Standard** | New components, bug fixes with tests, refactors with no behavior change, internal-API additions, single-domain features | Read every line of the diff. Run the change locally if it's UI. Check tests. ~10-30 minutes. |
| **L3 — Deep** | Cross-domain changes, public API changes, schema migrations, performance-critical paths, security-sensitive code, new external dependencies, financial-calculation logic | Read the diff. Read the surrounding code. Pull and run locally. Check the tests cover failure modes. Consider what could go wrong in production. ~30-90 minutes. May require multiple sessions. |
| **L4 — Architectural** | New service, new public package, new external integration, breaking change to a contract, change to error-handling strategy | Treat as a design review. May require an ADR or design doc to land first. Pair with a synchronous discussion if asynchronous review stalls. |

If you're unsure of the tier, default up. A 5-line change to a financial-calculation module is L3, not L1.

---

## 3. Self-Review (Before Requesting Review)

Run this before tagging anyone. The author is the first reviewer.

- [ ] CI is green. Every check passes locally too.
- [ ] The PR description states: what changed, why, how it was tested, any tradeoffs or follow-ups, links to issues/specs.
- [ ] The diff is minimal. No unrelated changes. No commented-out code. No `console.log` / `dbg!` / `print` left behind.
- [ ] You have read your own diff in the web UI. Inline comments left on non-obvious decisions.
- [ ] Tests exist and cover the new behavior plus its failure modes.
- [ ] No new `any`, `unwrap`, `expect`, non-null `!`, or silenced lints without an explanatory comment.
- [ ] Public surface changes (exports, types, function signatures) are intentional and documented.
- [ ] No secrets, API keys, customer data, or PII in the diff. Run a secret scanner if available.
- [ ] If the PR touches schema, types crossing trust boundaries, or shared packages: the corresponding consumers compile and tests pass.
- [ ] If the PR introduces a new dependency: it's justified in the description (see `coding-standards.md` §10).
- [ ] If the PR changes error semantics: callers handle the new error variants.
- [ ] If the PR is L3+ scope: tag the right reviewers and add a "what to focus on" note in the description.

PRs that fail self-review waste reviewer time. The bar is: a reviewer should never catch something the author would have caught by reading their own diff.

---

## 4. Reviewer Process

Read in this order. Each pass takes 30 seconds to a few minutes; the cumulative cost is small and the catch rate is high.

### Pass 1: Context (1-3 minutes)

- Read the PR description and linked issues.
- Open the relevant spec, design doc, or `CLAUDE.md` section if the change is non-trivial.
- Form a hypothesis: "This change should look approximately like X."

### Pass 2: Shape (1-2 minutes)

- Look at the file list. Are these the files you'd expect to change for this work?
- Surprises here are signals. A "fix typo" PR that touches twelve files needs explanation.
- Note the diff size against the tier from §2. L1 with 800 lines is a flag; L4 with 30 lines is a flag.

### Pass 3: Tests (3-10 minutes)

- Read the tests first. They tell you what behavior the author thinks is correct.
- Are the test names clear sentences? Do they cover happy path + failure modes + edge cases?
- For numerical or financial code: tolerance is specified, reference values are sourced.
- A PR with a passing CI but no new or modified tests is a flag — what behavior changed without test coverage?

### Pass 4: Implementation (the bulk)

- Read the diff in dependency order: types/schemas first, then leaf functions, then callers.
- Apply the checklist in §6.
- Leave comments inline. Be specific. Quote the line. Explain why, not just what.

### Pass 5: Mental simulation (2-5 minutes)

- Pick the worst-case path. What if this network call fails? What if the input is empty? What if two of these run concurrently? What if the user has 100k rows instead of 100?
- For UI: pull the branch and click around. Type into the form. Resize the window. Try keyboard navigation.
- For financial logic: pick a non-obvious case (zero quantity, negative cost basis, missing benchmark, leap year). Mentally trace it through.

### Pass 6: Approval decision

See §8. Default to approval if the diff is correct and tests pass; block only on substantive issues.

---

## 5. Comment Conventions

Prefix every comment with a tag so the author knows whether it blocks merge.

| Tag | Meaning | Author response |
|---|---|---|
| **blocker** | Must be addressed before merge. Correctness, security, broken contract. | Fix or push back with reasoning. |
| **issue** | Substantive concern. Default-blocking unless the author justifies the choice. | Fix or discuss. |
| **suggestion** | Recommendation. Author's call. | Address if convinced; ignore if not. |
| **question** | Reviewer wants to understand. Not necessarily a problem. | Answer in the thread. May lead to a code change or just shared understanding. |
| **nit** | Trivial. Style preference, micro-readability. | Author may ignore freely. Do not block on nits. |
| **praise** | Reviewer noting something well-done. | No action needed. Helpful for morale and signaling what to do more of. |

Examples:

```
blocker: this throws on empty input — line 42 will dereference undefined.
        add a guard or change the signature to accept the empty case.

issue: the retry policy here will fire for 4xx responses, which won't recover.
       guard on status >= 500 || status === 408.

suggestion: this would read better as an early return — the nested if/else
            obscures the happy path.

question: why throw rather than return Result here? the rest of the module
          is Result-based.

nit: this comment restates the function name; could probably go.

praise: nice use of property-based testing for the rebalance invariant —
        this would have been hard to cover with example-based tests.
```

The tag system has one job: reduce ambiguity about whether a comment blocks. Reviewers who write a wall of unblocking nits should consolidate them. Authors who treat every nit as a blocker should learn to discriminate.

---

## 6. What to Look For

The full checklist. Apply by category, not in order. Skip categories that don't apply.

### 6.1 Correctness

- Does it actually solve the stated problem?
- Edge cases: empty input, single element, duplicate keys, max size, zero, negative, NaN, very large numbers, very small numbers, Unicode, time zones, leap days, DST transitions.
- Off-by-one errors. Loop bounds. Slice indices.
- Floating-point comparisons (must use tolerance, never `===`).
- Integer overflow in any language without checked arithmetic by default.
- Race conditions. Anything that reads-then-writes shared state without a lock.
- Unbounded growth: collections that accumulate over time, timers that re-arm, subscriptions that never cancel.

### 6.2 Types and Schemas

- New `any`, `unknown` without narrowing, non-null assertions (`!`), `as` casts without type guards.
- `unwrap`, `expect`, or `panic!` in non-test Rust code.
- Public functions exported without explicit return types (TS) or with implicit `()` returns hiding error paths (Rust).
- Schema changes that aren't reflected in consumers.
- Discriminated unions without exhaustive switch / `assertNever` / `match`.

### 6.3 Error Handling

- What happens when this fails? Trace each external call (network, IPC, filesystem, db) and ask.
- Errors swallowed silently. Empty `catch` blocks. `.catch(() => {})`.
- String errors instead of typed error variants.
- Retry logic on non-retryable failures (validation, auth).
- Errors that include secrets, full file contents, or PII in the message.

### 6.4 Tests

- New behavior has new tests. New failure modes have new failure tests.
- Tests assert behavior, not implementation. A test that breaks on a refactor with no behavior change is a smell.
- Numerical tests specify tolerance.
- No `setTimeout` for synchronization in tests. No real network. No real clock.
- Property tests for invariants where applicable (sorting, allocation, accounting, round-trip serialization).
- Test names read as sentences and describe the behavior, not the function name.

### 6.5 Security

- All inputs from a trust boundary validated with a schema.
- Parameterized queries. No string interpolation into SQL, shell commands, or dynamic code.
- Secrets not committed. Secrets not logged. Secrets not in URLs or query strings.
- HTML/JSX rendering of user content does not bypass escaping.
- New external HTTP calls go through the project's HTTP client (timeouts, retries, observability).
- Any change to authentication, authorization, or session handling is L3 review minimum.

### 6.6 Concurrency

- TypeScript: every `await`, `then`, and async function is checked. Promises are awaited or explicitly `void`.
- Rust: every `.await` point in cancel-able code is cancel-safe. Locks released before `.await` unless intentional.
- No fire-and-forget without explicit intent.
- `Promise.all` vs `Promise.allSettled` chosen deliberately (fail-fast vs collect).
- Independent calls parallelized where safe.

### 6.7 API Design

- Public function signatures: minimal parameter count, named parameters via object for >3, no boolean flags (split into two functions or use a discriminated union).
- Return types: the smallest type that captures the contract. Don't return `unknown` or `any` from a public API.
- Naming: domain-language, predicate-shaped booleans, verb-shaped functions.
- Versioning: breaking changes to a public API require a deprecation cycle (see `architecture-principles.md` §13).

### 6.8 Performance

- Loop hot paths: any allocation, any I/O, any unnecessary work?
- Database: any N+1 query? Any scan-without-index?
- Bundle: any new heavy dependency imported eagerly? Code-split candidates left synchronous?
- Re-renders: any React component re-rendering on every keystroke when it shouldn't? Memoization where the cost of equality check outweighs the cost of re-render — that's worse than no memo.
- Cache invalidation: is it correct? Stale data is a correctness bug, not a performance bug.

### 6.9 Naming

- Names describe what the thing is in domain language.
- Functions read as verbs. Booleans read as predicates.
- Abbreviations only where universally recognized.
- No type suffixes (`fooObject`, `barClass`).

### 6.10 Documentation

- Public functions have doc comments explaining contract, not signature.
- Non-obvious tradeoffs commented inline with reasoning.
- TODO comments include owner and tracking link.
- README updated if behavior visible to users changed.

### 6.11 Scope

- The change is focused. Refactors are in separate commits or PRs from features.
- No drive-by fixes outside the PR's stated scope without an explanation.

---

## 7. What NOT to Comment On

If the linter or formatter would catch it, the linter or formatter should catch it. Don't review for:

- Whitespace, indentation, line length (formatter).
- Import ordering (linter).
- `let` vs `const` (linter).
- Single vs double quotes (formatter).
- Trailing commas (formatter).
- Curly brace style (formatter).
- Method ordering within a class (mostly noise).
- Whether to use `=>` or `function` for non-class methods (style).
- Whether a one-liner should be a multi-liner or vice versa (style).

If you find yourself wanting to leave a nit on any of the above, fix the linter config instead. Lint rules scale; reviewer attention does not.

---

## 8. Approval Decision

After the review passes, choose:

- **Approve** — diff is correct, tests pass, no blockers. Suggestions and nits left as comments; author handles at their discretion.
- **Approve with comments** — same as approve, but with non-blocking issues you'd like the author to consider. Often used when the diff is correct but you noticed adjacent code that could improve later.
- **Request changes** — at least one blocker or unresolved substantive issue.
- **Comment only** — used when you've read part of the diff and have observations but aren't taking a position yet. Useful for L3/L4 reviews that need multiple passes.

Default to approving. A culture where every PR gets "request changes" by default produces fearful authors who pad PRs to pre-empt nitpicks. The healthy ratio is most reviews approve, and "request changes" is reserved for substantive issues.

**Approving with concerns:** if you have non-blocking concerns but trust the author's judgment, approve with a comment noting the concern. Examples: "Approving — the retry policy here is fine for now, but we should revisit when we add the second provider." This trades velocity for follow-up burden, which is sometimes the right trade.

---

## 9. Handling Disagreement

When the author and reviewer disagree:

1. **Author responds first.** Either implement the change or push back with reasoning. Silence is not a response.
2. **Reviewer reads the response.** If convinced, resolve the thread. If not, restate the concern with new context.
3. **At impasse after one round-trip:** escalate to a third party (another engineer, a synchronous conversation, an ADR). Do not let a thread loop more than twice.
4. **Default: author's call.** The author is closest to the code. If the reviewer can't articulate a blocker, the author's choice stands.

The reviewer's role is to surface concerns and ask questions, not to win. If you find yourself digging in on a nit, you've lost the plot.

---

## 10. Special Review Types

### 10.1 Security-Sensitive

Authentication, authorization, cryptography, input validation at trust boundaries, secret handling, or anything touching customer data.

- Mandatory L3+ review.
- Two reviewers minimum where the team supports it.
- Apply OWASP Top 10 mentally as a checklist.
- Threat-model the change: what can an attacker do that they couldn't before?
- Run a secret scanner against the diff.
- Verify all new external inputs are schema-validated.

### 10.2 Performance-Critical

Hot paths, large-data paths, financial calculations, anything in a request critical path.

- Benchmarks before and after, ideally in the PR.
- Profile-driven justification for non-obvious choices.
- Allocation analysis for tight loops.
- Read query plans for new SQL.

### 10.3 Schema and Contract Changes

Database schema, public API types, IPC commands, event payloads, anything that crosses a versioning boundary.

- Backward compatibility analysis: what consumers exist? What breaks?
- Migration plan: how do existing rows / consumers transition?
- Versioning policy: is this a major bump? A deprecation? An additive change?
- Roll-forward and roll-back paths must both work.

### 10.4 Dependency Changes

Adding, upgrading, or removing a dependency.

- Justify the addition (see `coding-standards.md` §10).
- For upgrades: read the changelog. Note breaking changes.
- For removals: confirm no transitive consumer depends on it.
- License check.
- Bundle-size impact for frontend dependencies.

### 10.5 Financial / Numerical Code

Computation that produces numbers a human will trust for decisions.

- Reference values for tests come from a trusted source (NumPy, R, published example, hand-calculation). Source documented in the test.
- Tolerance specified explicitly.
- Edge cases: zero, negative, NaN, infinity, very large, very small, day-count edge cases (leap year, DST, market holidays).
- Currency: never store as float at rest. Use integer minor units or fixed-point decimal.
- Compounding direction (annualizing vs deannualizing) verified against a hand-calculation.

### 10.6 AI / Agent Code

Code that prompts, calls, or routes to LLMs or agentic systems.

- Prompt is in version control, not constructed from runtime data without sanitization.
- User input that flows into prompts is treated as untrusted (prompt injection vector).
- Tool/function calls validate arguments with a schema before execution.
- Token usage and cost telemetry exist for new model calls.
- Fallback path when the model returns malformed output (retry, schema-repair, or fail loudly — pick one explicitly).
- Tests use deterministic mocks for model calls; live model calls live in a separate eval suite.

---

## 11. AI-Assisted Review

When Claude or another AI assists in reviewing:

- AI surface-pass is supplementary, not authoritative. A human still owns the approval.
- Use AI for: catching common bugs (null derefs, missed error cases, unhandled promises), checking test coverage, suggesting alternative names, summarizing large diffs.
- Don't use AI for: architectural judgment, prioritization between tradeoffs, anything requiring context not in the diff.
- Treat AI suggestions like any other reviewer comment: tagged, weighed, and addressed or rejected with reasoning.
- An AI-generated review checklist run is fine; an AI-generated approval is not.

---

## 12. Author Responsibilities

The PR doesn't end at "submitted." The author owns:

- Responding to every comment, even nits (a thumbs-up emoji or "will do" is sufficient for trivial ones — though prefer not using emoji per project convention; "ack" or "fixed" is fine).
- Driving the PR to merge. Reviewers don't chase.
- Squashing or rebasing per project convention before merge.
- Watching CI on `main` for at least one cycle after merge — your change is your responsibility until the next deploy at minimum.
- Reverting promptly if the change causes a production issue. Roll-forward fixes after revert, not in lieu of it.

---

## 13. Anti-Patterns in Reviews

| Anti-pattern | Why it's bad | Better |
|---|---|---|
| Bikeshedding on style | Wastes attention on settled questions | Fix the linter config |
| Reviewer rewrites the PR in comments | Author loses ownership; cycle time explodes | Suggest direction, let author implement |
| "Drive-by" reviewers commenting without context | Comments are often misaligned with constraints the author knows | Read the description and linked context first |
| Approval without reading | Defeats the purpose | Decline to review if you don't have time |
| "Request changes" for nits | Trains authors to over-pad PRs | Use `nit:` and approve |
| Holding a PR hostage to an unrelated refactor | Conflates two changes | File a follow-up issue, approve the PR |
| Silent disagreement (ghosting a thread) | Blocks merge with no resolution | State a position or step away |
| "I would have done this differently" with no concrete suggestion | Unactionable | Either propose specifically or drop it |
| Reviewing without running, for changes that need running | Misses UX and runtime issues | Pull the branch for L3 UI changes |
| Approving your own PR's CI failure as "flaky" | Real flakes get masked | Investigate or file a flake ticket |

---

## 14. Review Etiquette

- Address the code, not the author. "This function does X" not "you do X".
- Lead with curiosity. "What was the reasoning for this approach?" before "this is wrong."
- Acknowledge tradeoffs the author already made. If they put a comment explaining why, engage with that comment.
- Praise good work, briefly. `praise:` tags exist for a reason.
- Time-box. If a review is taking more than 90 minutes, split it: leave what you have, finish later, or pair synchronously.
- Be timely. A PR sitting in review for >24 hours past first response is a process failure. If you can't review, decline so someone else can.

---

## 15. Hierarchy of Authority

When guidance conflicts:

1. **Project-specific `CLAUDE.md`** at the repository root.
2. **Project-specific docs** in `docs/`.
3. **`coding-standards.md`** (what good code looks like).
4. **`architecture-principles.md`** (what good design looks like).
5. **This file** (how reviews run).
6. **Ecosystem norms.**

The review process serves the code, not the other way around. If a rule here consistently causes friction with no benefit, raise it for revision.
