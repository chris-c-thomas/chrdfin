# Changelog & Release Instructions

This document is the canonical guide for Claude Code agents (and humans) working
on changelog entries, version bumps, and releases for chrdfin. Read it before
opening any release-related PR.

---

## Goals

1. **One human-readable changelog** at the repo root (`CHANGELOG.md`), in
   [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.
2. **Tag-driven releases** — only tagged commits produce installers and a
   GitHub Release. Pushing to `main` alone never ships anything.
3. **Phase-aligned version numbers** — pre-1.0, the minor version tracks the
   phase number. Stable v1.0.0 is reserved for the post-Phase-12 polish state.
4. **Automated entry collection** via [Changesets](https://github.com/changesets/changesets) —
   each PR adds a small markdown file, CI bundles them at release time.
5. **Cross-platform installers** built and attached to the GitHub Release by
   [`tauri-apps/tauri-action`](https://github.com/tauri-apps/tauri-action).

---

## Versioning scheme

### Pre-1.0 (current era)

| Tag | Meaning |
|---|---|
| `v0.0.1` | Phase 0 (foundation, scaffold, no user features). Already exists. |
| `v0.N.0` | Phase N completion (`N` ∈ 1..12). The minor version equals the phase number. |
| `v0.N.M` | Patch release within Phase N — bug fixes, small additions that don't warrant a phase bump. |
| `v1.0.0` | Stable release. Cut after Phase 12 lands and the app is feature-complete + battle-tested. |

**Why minor = phase:** keeps the tag history readable and avoids a private numbering scheme. `v0.5.0` always means "Phase 5 (Portfolio Tracker) shipped" without consulting a release note.

`v0.0.0` is not valid SemVer (the spec disallows it). Phase 0 is `v0.0.1`.

### Post-1.0

Standard SemVer kicks in:
- `MAJOR.MINOR.PATCH`
- Breaking changes → MAJOR bump
- New features → MINOR bump
- Bug fixes → PATCH bump

Pre-release channels (`-alpha.N`, `-beta.N`, `-rc.N`) become available if/when `dev` branch flow is adopted.

---

## Changelog format

Single root file: `CHANGELOG.md`. Sections per Keep a Changelog:

```markdown
## [0.1.0] - 2026-05-12

### Added
- Tiingo and FRED data adapters with background sync.
- `get_prices`, `search_tickers` Tauri commands.

### Changed
- Health check now reports last sync timestamp.

### Fixed
- Race condition in the schema initializer when DB file already existed.

### Removed

### Deprecated

### Security
```

Empty subsections can be omitted. Entries should describe **user-visible** behavior, not internal commit-level churn. "Fix typo in regex" doesn't go in the changelog; "Fix ticker search rejecting valid lowercase input" does.

The top of the file always has an `## [Unreleased]` section that accumulates pending entries between tagged releases. Changesets-driven releases populate this automatically.

---

## Branching model

**Trunk-based on `main` for now.** Single long-lived branch. Feature branches (`feat/phase-N-foo`) merge to `main` via PR. Tags cut from `main`.

Add a `dev` branch later **only** when one of these is true:

- You want pre-release channels (alphas, betas) without polluting `main`'s tag list.
- You have more than one regular contributor and PRs land faster than tags should fire.
- You go public and want a stable "released" view of the repo separate from in-flight work.

Until then, the rule is: **every commit on `main` must be releasable**. Failing CI on `main` is a fire drill, not a routine state.

---

## Tooling choices

| Tool | Role |
|---|---|
| **Changesets** | PR authors write `.changeset/*.md` files describing changes. CI aggregates into `CHANGELOG.md` and bumps versions. |
| **tauri-action** | GitHub Action that builds installers (`.dmg`, `.msi`, `.AppImage`, `.deb`) and attaches them to the GitHub Release tied to a `v*` tag. |
| **Keep a Changelog** | Human-readable changelog format. |
| **SemVer** | Version contract. |

Tools we explicitly **don't** use:

- `standard-version` / `release-please` — drives changelog from conventional commits, which aren't always at the right level for user-visible release notes.
- `cargo release` — Rust-side only; can't see the frontend.
- Manual changelog without Changesets — too easy to forget per PR.

---

## Repository structure (after setup)

```
chrdfin/
├── CHANGELOG.md                          # Root, human-readable, Keep a Changelog format
├── .changeset/
│   ├── config.json                       # Changesets config (fixed mode, desktop only)
│   ├── README.md                         # Changesets self-doc
│   └── *.md                              # One file per PR with pending entries
├── .github/workflows/
│   ├── ci.yml                            # Existing — typecheck/lint/test on every push
│   └── release.yml                       # NEW — runs on v* tags, builds installers
├── scripts/
│   └── sync-versions.ts                  # NEW — mirrors apps/desktop/package.json
│                                         # version into Cargo.toml files
└── apps/desktop/
    ├── package.json                      # Source of truth for the version
    └── src-tauri/Cargo.toml               # Mirrors the version (synced by script)
```

---

## Per-PR workflow

When opening a PR with user-visible changes:

```bash
pnpm exec changeset
```

The CLI prompts:

1. **Which packages?** — select `desktop` (the only versioned package; press space, enter).
2. **Bump type:**
   - `patch` — bug fix or trivial addition (no phase bump)
   - `minor` — new feature (phase bump if PR completes a phase)
   - `major` — breaking change (post-1.0 only; for pre-1.0 use minor)
3. **Summary** — one or two sentences in user-facing language. This becomes the changelog bullet verbatim, so write it like a release note.

This creates `.changeset/<random-name>.md`. Commit it as part of the PR.

If the PR is purely internal (refactor, doc-only, CI tweak), skip the changeset — it doesn't belong in a user-facing changelog. Convention: PR titles like `chore:` and `refactor:` typically don't need a changeset; `feat:` and `fix:` always do.

---

## Cutting a release

When ready to ship a tagged version:

```bash
# 1. Make sure main is clean and CI is green.
git checkout main && git pull

# 2. Apply pending changesets — bumps version, rewrites CHANGELOG.md, deletes the .changeset/*.md files it consumed.
pnpm exec changeset version

# 3. Sync the version into Cargo.toml files.
pnpm exec tsx scripts/sync-versions.ts

# 4. Review the diff. CHANGELOG.md and three version bumps should be the only changes.
git diff

# 5. Commit and tag.
git add -A
git commit -m "release: v0.N.M"
git tag v0.N.M
git push origin main --tags

# 6. CI takes over: release.yml fires on the tag, builds installers, drafts the GitHub Release.
```

Within the GitHub Release UI, paste the matching `CHANGELOG.md` section into the description (or let `tauri-action` extract it automatically — see config in `release.yml`).

---

## Phase-completion vs patch decisions

| Scenario | Action |
|---|---|
| Phase 1 (data layer) ships | `pnpm exec changeset` → minor bump → tag `v0.1.0` |
| Bug found in v0.1.0, fix merged | `pnpm exec changeset` → patch bump → tag `v0.1.1` |
| Phase 2 (compute core) ships while v0.1.x is still receiving patches | Tag `v0.2.0` from main; v0.1.x line is effectively dead unless a critical fix needs to be backported |
| A small new feature lands mid-phase | Bundle it into the next phase tag, don't cut a separate `v0.N.0` for it |
| Documentation-only changes | No changeset, no tag |

---

## When Claude is asked to "do a release"

The agent's job is **not** to push tags or create GitHub Releases — those require human review and (post-1.0) code-signing keys. The agent should:

1. Run `pnpm exec changeset version` and `pnpm exec tsx scripts/sync-versions.ts`.
2. Verify the diff is sensible (`CHANGELOG.md` + three version bumps, no other code changes).
3. Run the full verification gate: `pnpm typecheck`, `pnpm lint`, `pnpm format:check`, `pnpm test`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`.
4. Stage the changes, draft a `release: vX.Y.Z` commit message, and **stop** — do not commit or tag.
5. Report the proposed version, the changelog excerpt, and any anomalies for human review.

The human runs the final `git commit && git tag && git push --tags` after reviewing.

---

## Default git posture

The user prefers to run `git commit`, `git push`, `gh pr create`, and `git tag` themselves. Claude's default is to **prepare** the work (write code, stage files via `git add`, draft commit messages) and **report** — then provide the exact shell commands the user should run.

When the user explicitly asks Claude to commit/push/tag on their behalf:

- **All commits MUST be GPG-signed.** Use `git commit -S -m "..."` (or rely on `commit.gpgsign = true` in the user's git config — verify with `git config --get commit.gpgsign` before assuming it's on).
- **All tags MUST be signed.** Use `git tag -s vX.Y.Z -m "..."` for annotated signed tags. Never lightweight (`git tag vX.Y.Z`) — they can't be signed.
- If signing fails (no GPG key, expired key, no agent), stop and surface the error rather than committing unsigned. Don't pass `--no-gpg-sign`.
- Co-author trailers are off by default (the user has explicitly asked for no `Co-Authored-By:` lines on commit messages).

This applies even to `release: vX.Y.Z` commits and to all `v*` tags.

---

## When Claude is asked to "add a changeset"

For a feature/fix PR:

1. Determine bump type from the change scope (patch / minor; never major pre-1.0).
2. Write a one-or-two-sentence user-facing summary. Avoid commit-style language.
3. Create `.changeset/<descriptive-slug>.md` with the standard frontmatter:

   ```markdown
   ---
   "desktop": minor
   ---

   Summary line that will appear in the changelog verbatim.
   ```

4. Commit the file as part of the PR's work-in-progress.

---

## Implementation phases

This setup lands in two stages so the in-flight v0.0.1 PR isn't blocked.

### Step A — Hand-written `CHANGELOG.md` for v0.0.1

Already in scope for the immediate PR. Write `CHANGELOG.md` from the current commit history, tag the merge commit `v0.0.1` after the PR lands. No automation yet.

### Step B — Changesets + release workflow

Done before Phase 1 starts producing changelog churn. Concrete tasks:

1. `pnpm add -Dw @changesets/cli`
2. `pnpm exec changeset init`
3. Edit `.changeset/config.json`:
   ```json
   {
     "$schema": "https://unpkg.com/@changesets/config@3.0.0/schema.json",
     "changelog": ["@changesets/changelog-github", { "repo": "<owner>/chrdfin" }],
     "commit": false,
     "fixed": [["desktop"]],
     "linked": [],
     "access": "restricted",
     "baseBranch": "main",
     "updateInternalDependencies": "patch",
     "ignore": [
       "@chrdfin/charts",
       "@chrdfin/config",
       "@chrdfin/eslint-config",
       "@chrdfin/tsconfig",
       "@chrdfin/types",
       "@chrdfin/ui"
     ]
   }
   ```
   The `ignore` list keeps internal workspace packages out of the version flow — only the desktop app is versioned.
4. Add `.changeset/README.md` (auto-generated by `init` is fine).
5. Move the manual v0.0.1 entry from `CHANGELOG.md` into a Changesets-compatible format if needed (typically not — the existing `[0.0.1]` block stays where it is; Changesets only manages the `[Unreleased]` section going forward).
6. Add `scripts/sync-versions.ts` that reads `apps/desktop/package.json` and writes the same version to:
   - `apps/desktop/src-tauri/Cargo.toml` (`[package].version`)
   - `crates/chrdfin-core/Cargo.toml` (`[package].version`)
7. Add `.github/workflows/release.yml` triggered on `v*` tags. Uses `tauri-apps/tauri-action@v0` with platform matrix (`macos-latest`, `windows-latest`, `ubuntu-latest`). Output: draft GitHub Release with installers attached and the matching `CHANGELOG.md` section as the body.
8. Document the workflow in `CONTRIBUTING.md` (or a section in `README.md`).

---

## Code-signing & notarization (post-public)

The project is **private** until it's more feature-complete. Code signing is deferred to that public-launch milestone:

| Platform | What's needed |
|---|---|
| macOS | Apple Developer ID certificate + notarization with the `xcrun notarytool` workflow. Stored as GitHub Actions secrets. |
| Windows | Authenticode certificate (EV strongly preferred for SmartScreen reputation). |
| Linux | Optional GPG signing of `.deb` / `.AppImage`. |

Until that day, the `release.yml` produces unsigned installers tagged "developer build". Document this in the GitHub Release body so users (and you) aren't surprised by macOS Gatekeeper warnings.

Auto-update endpoints (Tauri's updater plugin) require a signing key pair generated separately. Defer to the same milestone.

---

## What NOT to do

- Do NOT push a `v*` tag without a corresponding entry in `CHANGELOG.md`.
- Do NOT manually edit `CHANGELOG.md` past releases (the `[0.0.1]`, `[0.1.0]` etc. blocks). Edits to historical entries break tooling and confuse readers.
- Do NOT use `--force` on tags. If a tag is wrong, cut a new patch (`v0.N.M+1`) with a "fixes mistakenly tagged v0.N.M" entry and retract the bad release on GitHub.
- Do NOT publish workspace packages (`@chrdfin/*`) to npm. They're internal. Keep `"private": true` on every `package.json` except the root.
- Do NOT add changesets for purely internal refactors, doc-only PRs, or tooling tweaks. They make the changelog noisy.
- Do NOT skip the `cargo` checks before tagging. The release workflow assumes the workspace is green.

---

## References

- [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
- [SemVer](https://semver.org/)
- [Changesets docs](https://github.com/changesets/changesets/blob/main/docs/intro-to-using-changesets.md)
- [tauri-action](https://github.com/tauri-apps/tauri-action)
- [Tauri updater plugin](https://v2.tauri.app/plugin/updater/)
