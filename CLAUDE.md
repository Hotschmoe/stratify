# CLAUDE.md — Stratify

## RULE 1 – ABSOLUTE (DO NOT EVER VIOLATE THIS)

You may NOT delete any file or directory unless I explicitly give the exact command **in this session**.

- This includes files you just created (tests, tmp files, scripts, etc.).
- You do not get to decide that something is "safe" to remove.
- If you think something should be removed, stop and ask. You must receive clear written approval **before** any deletion command is even proposed.

Treat "never delete files without permission" as a hard invariant.

---

### IRREVERSIBLE GIT & FILESYSTEM ACTIONS

Absolutely forbidden unless I give the **exact command and explicit approval** in the same message:

- `git reset --hard`
- `git clean -fd`
- `rm -rf`
- Any command that can delete or overwrite code/data

Rules:

1. If you are not 100% sure what a command will delete, do not propose or run it. Ask first.
2. Prefer safe tools: `git status`, `git diff`, `git stash`, copying to backups, etc.
3. After approval, restate the command verbatim, list what it will affect, and wait for confirmation.
4. When a destructive command is run, record in your response:
   - The exact user text authorizing it
   - The command run
   - When you ran it

If that audit trail is missing, then you must act as if the operation never happened.

---

### Code Editing Discipline

- Do **not** run scripts that bulk-modify code (codemods, invented one-off scripts, giant `sed`/regex refactors).
- Large mechanical changes: break into smaller, explicit edits and review diffs.
- Subtle/complex changes: edit by hand, file-by-file, with careful reasoning.
- **NO EMOJIS** - do not use emojis or non-textual characters.
- ASCII diagrams are encouraged for visualizing flows.
- Keep in-line comments to a minimum. Use external documentation for complex logic.
- In-line commentary should be value-add, concise, and focused on info not easily gleaned from the code.

---

### Backwards Compatibility & File Sprawl

We optimize for a clean architecture, not backwards compatibility.

- No "compat shims" or "v2" file clones.
- When changing behavior, migrate callers and remove old code **inside the same file**.
- New files are only for genuinely new domains that don't fit existing modules.
- The bar for adding files is very high.

---

## Beads (bd) - Task Management

Beads is a git-backed graph issue tracker. Use `--json` flags for all programmatic operations.

### Session Workflow

```
1. bd prime              # Auto-injected via SessionStart hook
2. bd ready --json       # Find unblocked work
3. bd update <id> --status in_progress --json   # Claim task
4. (do the work)
5. bd close <id> --reason "Done" --json         # Complete task
6. bd sync && git push   # End session - REQUIRED
```

### Key Commands

| Action | Command |
|--------|---------|
| Find ready work | `bd ready --json` |
| Find stale work | `bd stale --days 30 --json` |
| Create issue | `bd create "Title" --description="Context" -t bug\|feature\|task -p 0-4 --json` |
| Create discovered work | `bd create "Found bug" -t bug -p 1 --deps discovered-from:<parent-id> --json` |
| Claim task | `bd update <id> --status in_progress --json` |
| Complete task | `bd close <id> --reason "Done" --json` |
| Find duplicates | `bd duplicates` |
| Merge duplicates | `bd merge <id1> <id2> --into <canonical> --json` |

### Critical Rules

- Always include `--description` when creating issues - context prevents rework
- Use `discovered-from` links to connect work found during implementation
- Run `bd sync` at session end before pushing to git
- **Work is incomplete until `git push` succeeds**
- `.beads/` is authoritative state and **must always be committed** with code changes

### Dependency Thinking

Use requirement language, not temporal language:
```bash
bd dep add rendering layout      # rendering NEEDS layout (correct)
# NOT: bd dep add phase1 phase2   (temporal - inverts direction)
```

### After bd Upgrades

```bash
bd info --whats-new              # Check workflow-impacting changes
bd hooks install                 # Update git hooks
bd daemons killall               # Restart daemons
```

---

## Beads Viewer (bv) - Triage Engine

bv is a graph-aware triage engine for Beads projects. It provides precomputed metrics (PageRank, betweenness, critical path, cycles) without parsing JSONL or hallucinating graph traversal.

> [!CAUTION]
> Use ONLY `--robot-*` flags. Bare `bv` launches an interactive TUI that blocks your session.

### Entry Point

**`bv --robot-triage` is the single entry point.** It returns:
- `quick_ref`: at-a-glance counts + top 3 picks
- `recommendations`: ranked actionable items with scores
- `quick_wins`: low-effort high-impact items
- `blockers_to_clear`: items that unblock the most downstream work
- `commands`: copy-paste shell commands for next steps

```bash
bv --robot-triage    # THE MEGA-COMMAND: start here
bv --robot-next      # Minimal: just the single top pick
```

### Robot Commands

| Command | Returns |
|---------|---------|
| `--robot-triage` | Full project snapshot with ranked recommendations |
| `--robot-next` | Top priority item with claim command |
| `--robot-plan` | Parallel execution tracks with `unblocks` lists |
| `--robot-insights` | Full metrics: PageRank, betweenness, cycles, critical path |
| `--robot-priority` | Priority misalignment detection |
| `--robot-alerts` | Stale issues, blocking cascades, priority mismatches |
| `--robot-suggest` | Hygiene: duplicates, missing deps, cycle breaks |
| `--robot-history` | Bead-to-commit correlations and event timelines |
| `--robot-forecast <id\|all>` | Dependency-aware ETA predictions |

### Filtering & Scoping

```bash
bv --robot-plan --label backend           # Scope to label
bv --recipe actionable --robot-plan       # Pre-filter: ready to work
bv --recipe high-impact --robot-triage    # Top PageRank items
bv --robot-triage --robot-triage-by-track # Group by parallel work streams
bv --robot-triage --robot-triage-by-label # Group by domain
```

### Graph Export

```bash
bv --robot-graph --graph-format=mermaid   # Mermaid diagram
bv --robot-graph --graph-format=dot       # Graphviz DOT
bv --export-graph graph.html              # Interactive HTML
```

### jq Quick Reference

```bash
bv --robot-triage | jq '.quick_ref'              # At-a-glance summary
bv --robot-triage | jq '.recommendations[0]'     # Top recommendation
bv --robot-plan | jq '.plan.summary.highest_impact'  # Best unblock target
bv --robot-insights | jq '.Cycles'               # Circular deps (must fix!)
```

### Two-Phase Analysis

- **Phase 1 (instant):** degree, topo sort, density - always available
- **Phase 2 (async, 500ms timeout):** PageRank, betweenness, cycles - check `status` flags

For large graphs (>500 nodes), some metrics may be approximated. Always check `status` in output.

---

## Session Completion Checklist

```
[ ] File issues for remaining work (bd create)
[ ] Run quality gates (tests, linters)
[ ] Update issue statuses (bd update/close)
[ ] Run bd sync
[ ] Run git push and verify success
[ ] Confirm git status shows "up to date"
```

**Work is not complete until `git push` succeeds.**

---

## Claude Agents

Two specialized agents are available in `.claude/agents/`:

### coder-sonnet

Fast code implementer for precise execution of tasks. Use for:
- Quick, targeted code changes
- Following existing patterns and conventions
- Atomic commits with clear descriptions

### gemini-analyzer

Delegates large-context analysis to Gemini CLI. Use for:
- Large pattern recognition across the codebase
- Architecture analysis requiring 1M+ context
- Full repository scans

Commands:
```bash
gemini --all-files -p "prompt here"  # Full repo scan
gemini -p "prompt here"              # Specific prompt
```

---

## Contribution Policy

Remove any mention of contributing/contributors from README and don't reinsert it.

---

# PROJECT-LANGUAGE-SPECIFIC: Rust

> **To adapt for another language:** Replace this entire section with language-specific toolchain, testing, best practices, and code search examples.

## Rust Toolchain

- **Rust Edition**: 2021 (check `Cargo.toml` for exact version)
- Build: `cargo build` (add `--release` for optimized builds)
- Test: `cargo test` (185 tests: 149 unit + 36 doc)
- Format: `cargo fmt` (run before commits)
- Lint: `cargo clippy`

### Key Commands

```bash
# Build and run GUI (native)
cargo run --bin calc_gui

# Build release
cargo build --release --bin calc_gui

# Run all tests
cargo test

# Build CLI (placeholder only)
cargo run --bin calc_cli

# Build and run WebAssembly (working with WebGPU)
rustup target add wasm32-unknown-unknown
cd calc_gui && trunk serve --open
```

### Cargo Workspace

- Lockfile: `Cargo.lock` (auto-managed by Cargo)
- Dependencies: Defined in workspace `Cargo.toml`
- Clean: `cargo clean` to remove build artifacts

---

## Logging & Console Output

- Use structured logging with `log` or `tracing` crates; avoid raw `println!` for production.
- GUI output goes through Iced; CLI through ratatui if needed.
- Errors should use `thiserror` or `anyhow` for context and chaining.

---

## Third-Party Libraries

When unsure of an API, look up current docs rather than guessing. Key dependencies:

- **iced 0.14**: GUI framework (Elm architecture) with canvas support
- **ratatui**: TUI framework for CLI interfaces
- **typst 0.14 + typst-pdf**: PDF generation with BerkeleyMono font
- **serde / serde_json**: Serialization (JSON for `.stf` project files)
- **fs2**: Cross-platform file locking for NAS/cloud drive safety
- **rfd**: Native file dialogs
- **wgpu 27.0**: WebGPU for native and WASM rendering
- **trunk**: WASM build tool for browser deployment

---

## Testing Guidelines

### Test Commands

```bash
cargo test                              # All tests (185: 149 unit + 36 doc)
cargo test -- --nocapture               # Show println! output
cargo test beam_                        # Run tests matching pattern
cargo test --doc                        # Doc tests only
cargo test -p calc_core                 # Tests for specific crate
```

### Test Patterns

- Use `#[test]` attribute for unit tests
- Use `#[cfg(test)]` modules for test-only code
- Use `tempfile` crate for temporary files
- Use `assert!`, `assert_eq!`, `assert_ne!` macros
- Doc tests: code blocks in `///` comments are tested automatically

---

## Rust Best Practices

### Error Handling

```rust
// Use Result and the ? operator for propagation
fn load_config() -> Result<Config, CalcError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

// Use .context() with anyhow for additional context
use anyhow::Context;
let data = read_file().context("failed to load project")?;
```

### Option Handling

```rust
// Prefer pattern matching or combinators over unwrap()
if let Some(item) = items.get(index) {
    // safe to use item
}

// Use unwrap_or, unwrap_or_default, or unwrap_or_else
let value = optional.unwrap_or(default_value);
```

### Division Safety

```rust
// Guard against division by zero
let avg = if !items.is_empty() {
    total / items.len() as f64
} else {
    0.0
};
```

### Ownership & Borrowing

```rust
// Prefer borrowing over cloning when possible
fn process(data: &BeamInput) -> BeamResult { ... }

// Use Cow<str> for flexible string ownership
use std::borrow::Cow;
fn process_name(name: Cow<'_, str>) { ... }
```

---

## Code Search Tools

### ripgrep (rg) — Fast Text Search

Use when searching for literal strings or regex patterns:

```bash
rg "unwrap()" -t rust              # Find all unwrap() calls
rg "TODO|FIXME" -t rust            # Find todos
rg "pub fn" src/                   # Find public functions
rg -l "BeamInput" -t rust          # List files containing pattern
rg -n "impl.*Display" -t rust      # Show line numbers
```

### ast-grep — Structural Code Search

Use when you need syntax-aware matching (ignores comments/strings, understands code structure):

```bash
ast-grep run -l rust -p '$X.unwrap()'        # Find all unwrap() calls
ast-grep run -l rust -p 'panic!($$$)'        # Find all panic! macros
ast-grep run -l rust -p 'fn $NAME() { $$$ }' # Find functions with no params
ast-grep run -l rust -p 'clone()'            # Find all clone() calls
```

**When to use which:**
- **ripgrep**: Quick searches, TODOs, config values, recon
- **ast-grep**: Refactoring patterns, finding anti-patterns, policy checks

---

## Bug Severity (Rust-specific)

- **Critical**: `unwrap()` on None/Err in production paths, division by zero, panics, memory leaks (e.g., `Rc` cycles)
- **Important**: Missing error propagation (`?`), `clone()` where borrow would suffice, unsafe blocks without justification
- **Contextual**: TODO/FIXME, unused imports, dead code warnings

---

we love you, Claude! do your best today