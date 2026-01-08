# CLAUDE.md - Stratify

## RULE 1 - ABSOLUTE (DO NOT EVER VIOLATE THIS)

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

### Version Updates (SemVer)

When making commits, update the `version` in `Cargo.toml` (workspace root) following [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0): Breaking changes or incompatible API modifications
- **MINOR** (0.X.0): New features, backward-compatible additions
- **PATCH** (0.0.X): Bug fixes, small improvements, documentation

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

### No Legacy Code - Full Migrations Only

We optimize for clean architecture, not backwards compatibility. **When we refactor, we fully migrate.**

- No "compat shims", "v2" file clones, or deprecation wrappers
- When changing behavior, migrate ALL callers and remove old code **in the same commit**
- No `_legacy` suffixes, no `_old` prefixes, no "will remove later" comments
- New files are only for genuinely new domains that don't fit existing modules
- The bar for adding files is very high

**Rationale**: Legacy compatibility code creates technical debt that compounds. A clean break is always better than a gradual migration that never completes.

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

### Context Preservation During Debugging

Long debugging sessions can lose context during compaction. **Commit frequently to preserve investigation state.**

```bash
# During debugging - commit investigation findings periodically
git add -A && git commit -m "WIP: investigating X, found Y"
bd create "Discovered: Z needs fixing" -t bug -p 2 --description="Found while debugging X"
bd sync

# At natural breakpoints (every 30-60 min of active debugging)
bd sync  # Capture bead state changes
git push  # Push to remote
```

**Why this matters:**
- Compaction events lose conversational context but git history persists
- Beads issues survive across sessions - use them to capture findings
- "WIP" commits are fine - squash later when the fix is complete
- A partially-documented investigation beats starting over

---

## Session Completion Checklist

```
[ ] File issues for remaining work (bd create)
[ ] Run quality gates (cargo test, cargo clippy)
[ ] Update issue statuses (bd update/close)
[ ] Run bd sync
[ ] Run git push and verify success
[ ] Confirm git status shows "up to date"
```

**Work is not complete until `git push` succeeds.**

---

## Claude Agents

Specialized agents are available in `.claude/agents/`. Agents use YAML frontmatter format:

```yaml
---
name: agent-name
description: What this agent does
model: sonnet|haiku|opus
tools:
  - Bash
  - Read
  - Edit
---
```

### Available Agents

| Agent | Model | Purpose |
|-------|-------|---------|
| coder-sonnet | sonnet | Fast, precise code changes with atomic commits |
| build-verifier | sonnet | Validates all targets (native + WASM) compile |
| gemini-analyzer | sonnet | Large-context analysis via Gemini CLI (1M+ context) |

### Disabling Agents

To disable specific agents in `settings.json` or `--disallowedTools`:
```json
{
  "disallowedTools": ["Task(build-verifier)", "Task(gemini-analyzer)"]
}
```

---

## Claude Skills

Skills are invoked via `/skill-name`. Available in `.claude/skills/`.

### Skill Frontmatter (v2.1+)

Skills now support YAML frontmatter with advanced options:

```yaml
---
name: skill-name
description: What this skill does
# Run in forked sub-agent context (isolated from main conversation)
context: fork
# Specify which agent executes this skill
agent: coder-sonnet
---
```

| Field | Description |
|-------|-------------|
| `context: fork` | Run skill in isolated sub-agent context |
| `agent: <name>` | Execute skill using specified agent type |

### Built-in Commands

| Command | Purpose |
|---------|---------|
| `/plan` | Enter plan mode for implementation design |
| `/context` | Manage context files and imports |
| `/help` | Show available commands |

### Project Skills

| Skill | Purpose |
|-------|---------|
| `/test` | Run cargo test with optional filtering |
| `/check` | Run cargo check + clippy on all targets |

### Skill Hot-Reload

Skills in `.claude/skills/` are automatically discovered without restart. Edit or add skills and they become immediately available.

---

# PROJECT-LANGUAGE-SPECIFIC: Rust (Edition 2021)

## Project Overview

Stratify is a structural engineering calculation application for wood and steel design. It provides:

- **calc_core**: Core calculation library (beams, columns, materials, NDS factors, PDF generation)
- **calc_gui**: Iced-based GUI application (native + WASM/WebGPU)
- **calc_cli**: Ratatui-based terminal UI

**Key principle**: Clean, LLM-friendly API. All types are JSON-serializable via serde for integration with AI assistants.

---

## Rust Toolchain

- **Rust Edition**: 2021
- **Targets**: Native (Windows/Mac/Linux) + WASM (wasm32-unknown-unknown)
- **GUI**: Iced 0.14 with wgpu backend
- **TUI**: Ratatui + crossterm
- **PDF**: Typst integration

### Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run GUI
cargo run --bin calc_gui

# Run CLI
cargo run --bin calc_cli

# Run tests
cargo test

# Check all targets (fast feedback)
cargo check --all-targets

# Clippy lints
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt

# WASM build (requires wasm-pack or trunk)
cargo build --target wasm32-unknown-unknown -p calc_gui
```

---

## Architecture

### Key Directories

```
stratify/
  calc_core/           # Core calculation library
    src/
      calculations/    # Beam, column, continuous beam analysis
      equations/       # Documented statics formulas
      materials/       # Material definitions (lumber, steel)
      loads/           # Load types, ASCE 7 combinations
      generated/       # Build-time generated data (materials from TOML)
      pdf.rs           # PDF generation via Typst
      project.rs       # Project container and metadata
      file_io.rs       # Atomic file operations with locking
  calc_gui/            # Iced GUI application
    src/
      ui/              # UI components (panels, modals, inputs)
      update.rs        # Update checking logic
  calc_cli/            # Ratatui TUI application
  data/                # TOML data files for code generation
```

### Design Principles

- **Stateless calculations**: Pure functions that take input and return results
- **JSON-first**: All types implement `Serialize`/`Deserialize`
- **Rich errors**: Structured error types via `thiserror`, not strings
- **Generated data**: Material databases generated at build time from TOML

---

## Rust Best Practices

### Error Handling

```rust
// Use thiserror for structured errors
#[derive(Debug, thiserror::Error)]
pub enum CalcError {
    #[error("Invalid span: {0} must be positive")]
    InvalidSpan(f64),
    #[error("Material not found: {0}")]
    MaterialNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Use Result<T, E> consistently
pub fn analyze_beam(input: BeamInput) -> Result<BeamResult, CalcError> {
    if input.span <= 0.0 {
        return Err(CalcError::InvalidSpan(input.span));
    }
    // ...
}

// Use ? for propagation
pub fn load_and_analyze(path: &Path) -> Result<BeamResult, CalcError> {
    let project = load_project(path)?;  // Propagates IO errors
    let input = project.get_beam_input()?;
    analyze_beam(input)
}
```

### Option Handling

```rust
// Prefer match/if-let over .unwrap()
if let Some(material) = materials.get(&name) {
    // Use material
} else {
    return Err(CalcError::MaterialNotFound(name));
}

// Use .unwrap_or_default() or .unwrap_or() for safe defaults
let deflection_limit = config.deflection_limit.unwrap_or(360.0);

// Use .ok_or() to convert Option to Result
let material = materials.get(&name)
    .ok_or_else(|| CalcError::MaterialNotFound(name.clone()))?;
```

### Struct Design

```rust
// Use derive macros consistently
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeamInput {
    pub span: f64,           // feet
    pub width: f64,          // inches
    pub depth: f64,          // inches
    pub material: Material,
    pub loads: Vec<LoadCase>,
}

// Use builder pattern for complex construction
impl BeamInput {
    pub fn new(span: f64, width: f64, depth: f64) -> Self {
        Self {
            span,
            width,
            depth,
            material: Material::default(),
            loads: Vec::new(),
        }
    }

    pub fn with_material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }
}
```

### Module Organization

```rust
// In mod.rs or lib.rs - re-export public API
pub mod calculations;
pub mod materials;

// Re-export commonly used types at crate root
pub use calculations::{BeamResult, ColumnResult};
pub use materials::Material;

// Keep internal helpers private
mod internal_utils;  // Not `pub mod`
```

### Iterators and Closures

```rust
// Prefer iterator chains over manual loops
let max_moment = load_cases.iter()
    .map(|lc| calculate_moment(lc, span))
    .fold(0.0, f64::max);

// Use collect with turbofish for type inference
let valid_loads: Vec<_> = loads.iter()
    .filter(|l| l.magnitude > 0.0)
    .collect();

// Use for loops when side effects are needed
for load in &mut loads {
    load.apply_factor(factor);
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beam_moment_simple_span() {
        let input = BeamInput::new(10.0, 3.5, 9.25);
        let result = analyze_beam(input).unwrap();

        // Use approx comparisons for floats
        assert!((result.max_moment - 12500.0).abs() < 0.1);
    }

    #[test]
    fn test_invalid_span_returns_error() {
        let input = BeamInput::new(-5.0, 3.5, 9.25);
        assert!(matches!(
            analyze_beam(input),
            Err(CalcError::InvalidSpan(_))
        ));
    }
}
```

### Documentation

```rust
/// Analyzes a simply-supported beam under the given loads.
///
/// # Arguments
///
/// * `input` - Beam geometry, material, and load cases
///
/// # Returns
///
/// Beam analysis results including moments, shears, and deflections.
///
/// # Errors
///
/// Returns `CalcError::InvalidSpan` if span is not positive.
///
/// # Example
///
/// ```
/// let input = BeamInput::new(12.0, 3.5, 11.25);
/// let result = analyze_beam(input)?;
/// println!("Max moment: {} lb-ft", result.max_moment);
/// ```
pub fn analyze_beam(input: BeamInput) -> Result<BeamResult, CalcError> {
    // ...
}
```

---

## GUI Development (Iced)

### Message Pattern

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // User actions
    SpanChanged(String),
    MaterialSelected(Material),
    Calculate,

    // Async results
    CalculationComplete(Result<BeamResult, CalcError>),
    UpdateCheckComplete(Option<Version>),
}

fn update(&mut self, message: Message) -> Command<Message> {
    match message {
        Message::SpanChanged(s) => {
            self.span_input = s;
            Command::none()
        }
        Message::Calculate => {
            let input = self.build_input();
            Command::perform(
                async move { analyze_beam(input) },
                Message::CalculationComplete
            )
        }
        // ...
    }
}
```

### View Composition

```rust
fn view(&self) -> Element<Message> {
    let content = column![
        self.view_toolbar(),
        row![
            self.view_input_panel(),
            self.view_results_panel(),
        ],
        self.view_status_bar(),
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
```

---

## WASM Considerations

```rust
// Use cfg attributes for platform-specific code
#[cfg(not(target_arch = "wasm32"))]
fn save_file(path: &Path, data: &[u8]) -> io::Result<()> {
    fs::write(path, data)
}

#[cfg(target_arch = "wasm32")]
fn save_file(_path: &Path, data: &[u8]) -> io::Result<()> {
    // Use web-sys to trigger browser download
    trigger_download(data)
}

// fs2 (file locking) is native-only
#[cfg(not(target_arch = "wasm32"))]
use fs2::FileExt;
```

---

## Testing Guidelines

### Test Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_beam_moment

# Run tests in specific crate
cargo test -p calc_core

# Run ignored (slow) tests
cargo test -- --ignored
```

### Test Organization

- Unit tests go in the same file as the code (`#[cfg(test)] mod tests`)
- Integration tests go in `tests/` directory
- Use `proptest` or `quickcheck` for property-based testing when appropriate

---

## Common Development Workflows

### Adding a New Calculation Type

1. **Define types** in `calc_core/src/calculations/`
   - Input struct with `Serialize`/`Deserialize`
   - Result struct with calculated values
   - Error variants if needed

2. **Implement calculation** as pure function
   - Take input, return `Result<Output, CalcError>`
   - No side effects, no global state

3. **Add tests** in same file or `tests/`

4. **Wire up GUI** in `calc_gui/src/ui/`
   - Input panel component
   - Results panel component
   - Message variants for user actions

5. **Export from lib.rs** if public API

### Adding a New Material

1. Add entry to `data/materials.toml`
2. Rebuild to regenerate `calc_core/src/generated/material_data.rs`
3. Material is automatically available via `Material::lookup()`

---

## Bug Severity (Rust)

### Critical - Must Fix Immediately

- `.unwrap()` or `.expect()` on user input (panic in release)
- Index out of bounds (`slice[i]` without bounds check)
- Integer overflow in release builds
- Deadlocks in async code
- Memory leaks (though rare in Rust)

### Important - Fix Before Merge

- Missing error handling (using `.unwrap()` where `?` should be used)
- Clippy warnings (especially `clippy::pedantic` findings)
- Inconsistent public API (missing `pub` or wrong visibility)
- Missing documentation on public items
- Non-idiomatic code patterns

### Contextual - Address When Convenient

- TODO/FIXME comments
- Unused imports or variables (compiler warns)
- Suboptimal iterator usage
- Missing `#[must_use]` on functions returning Result

---

## Development Philosophy

**Make it work, make it right, make it fast** - in that order.

- Use `dbg!()` macro during development, remove before commit
- Run `cargo clippy` before every commit
- Run `cargo fmt` to maintain consistent style
- Keep dependencies minimal - prefer std library when possible

**The goal**: A clean, well-documented calculation library that both humans and LLMs can understand and extend.

---

we love you, Claude! do your best today
