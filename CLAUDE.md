# CLAUDE.md - Laminae

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

### Version Updates (SemVer)

When making commits, update the `version` in `build.zig.zon` following [Semantic Versioning](https://semver.org/):

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
[ ] Run quality gates (tests, linters)
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
| gemini-analyzer | sonnet | Large-context analysis via Gemini CLI (1M+ context) |
| build-verifier | sonnet | Pre-merge validation for all 4 kernel variants |
| regression-detector | haiku | Smoke token comparison against baseline |
| dtb-analyzer | sonnet | Device tree analysis for driver development |

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
| `/test` | Run test_all.zig with smart variant selection |
| `/verify` | Build all variants and check for regressions |
| `/symbolize <addr>` | Resolve kernel address to function name |
| `/syscall` | Guided syscall addition workflow |
| `/stack-check` | Analyze kernel stack usage patterns |

### Skill Hot-Reload

Skills in `.claude/skills/` are automatically discovered without restart. Edit or add skills and they become immediately available.

---

# PROJECT-LANGUAGE-SPECIFIC: Zig (v0.15.2)

> **Freestanding kernel**: This is a bare-metal OS kernel. There are NO third-party libraries or package manager dependencies. All code is self-contained.

## Project Overview

Laminae is a container-native research kernel for ARM (AArch64). Containers are first-class OS citizens - the kernel sees Container IDs, not PIDs. Each container owns a hardware-enforced slice of virtual address space with ASID-based isolation.

**Key principle**: We are not recreating Linux. This is a research platform exploring what a container-native kernel looks like without 30 years of process-model legacy.

---

## Zig Toolchain

- **Zig Version**: 0.15.2 (freestanding target)
- **Target**: `aarch64-freestanding-none`
- Build: `zig build` (see build options below)
- Format: `zig fmt` (run before commits)
- No external package manager - all code is self-contained

### Build Commands

```bash

# Build with specific variant
zig build -Dvariant=debug     # Debug build (default)
zig build -Dvariant=fast      # ReleaseFast
zig build -Dvariant=safe      # ReleaseSafe
zig build -Dvariant=small     # ReleaseSmall

# Test all kernel variants (regression testing)
zig run scripts/test_all.zig -- --timeout=5000
```

### Build Options

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `-Dvariant` | debug, fast, safe, small | debug | Kernel optimization level |
| `-Dboot_target` | ci, interactive | ci | Boot target: ci (test_harness) or interactive (shell) |
| `-Dearly_uart_debug` | true/false | false | Hardcoded UART in EL2 for early debugging |

---

## Architecture

### Exception Levels

- **EL2**: Initial boot, MMU setup, GIC/timer init, then transitions to EL1
- **EL1**: Kernel core only - scheduler, syscall dispatch, memory management
- **EL0**: All containers run here with hardware privilege separation

### Key Directories

- `src/kernel/` - Kernel subsystems (container/, syscalls/, memory/, drivers/)
- `src/kernel/syscalls/table.zig` - **THE source of truth** for all syscall definitions
- `src/kernel/container/table.zig` - Container type definitions and capabilities
- `src/shared/` - Code shared between kernel and user-space (compat/table.zig, icc/schema.zig)
- `user/programs/` - User application containers
- `user/drivers/` - Hardware driver containers (netd.zig, genetd.zig, blkd.zig)
- `user/services/` - System services (lamina.zig orchestrator, shell.zig)
- `user/liblaminae/` - **GENERATED** user-space library (syscall wrappers, net_api)

### Source of Truth Architecture

The kernel uses **table-driven code generation** to maintain consistency across the stack:

```
src/kernel/syscalls/table.zig     ->  zig build gen-lib  ->  user/liblaminae/syscalls.zig
src/kernel/container/table.zig    ->  zig build gen-docs ->  docs/api/syscalls.md
src/shared/compat/table.zig       ->  (shared directly)
src/shared/icc/schema.zig         ->  (shared directly)
```

**Critical**: When adding or modifying syscalls:
1. Edit `src/kernel/syscalls/table.zig` (the spec array with name, num, args, etc.)
2. Run `zig build gen-lib` to regenerate liblaminae syscall wrappers
3. Run `zig build gen-docs` to regenerate API documentation
4. The build system auto-runs `gen-lib` before compiling user programs

**Never manually edit** `user/liblaminae/syscalls.zig` - it is overwritten on every build.

---

## Print System & Console Output

- **EL2 (early boot)**: Use `uart.puts()` only - no `printz()`/`debugz()`
- **EL1/EL0 (after MMU)**: Use `printz()` for production, `debugz()` for dev scaffolding

```zig
// EL2 - CORRECT (before MMU is active)
uart.puts("MMU: ");
uart.putDec(total_mb, null);

// EL1/EL0 - CORRECT (after MMU is active)
printz("Container {} ready\n", .{id});
debugz("Debug info: {x}\n", .{addr});  // Stripped in production
```

### Global Primitives

Access via `@import("root")`:
- `barriers` - ARM memory barriers (dmb/dsb/isb)
- `cache` - Cache operations
- `printz`, `debugz`, `panicz` - Print functions

---

## Zig Best Practices (Freestanding)

### Error Handling

```zig
// Use error unions and try for propagation
fn loadConfig(path: []const u8) !Config {
    const file = try fs.open(path);
    defer file.close();
    return try parseConfig(file);
}

// Explicit error sets for API boundaries
const ConfigError = error{
    FileNotFound,
    ParseFailed,
    InvalidFormat,
};

fn parseConfig(data: []const u8) ConfigError!Config {
    // ...
}
```

### Optional Handling

```zig
// Prefer if/orelse over .? when handling is needed
if (items.get(index)) |item| {
    // safe to use item
} else {
    // handle missing case
}

// Use orelse for defaults
const value = optional orelse default_value;

// Use .? only when null is truly unexpected
const ptr = maybe_ptr.?;  // Will panic if null - use sparingly
```

### Memory Safety

```zig
// Always use defer for cleanup
const buffer = try allocator.alloc(u8, size);
defer allocator.free(buffer);

// Prefer slices over raw pointers
fn process(data: []const u8) void { ... }

// Use sentinel-terminated slices for C interop
fn cString(s: [:0]const u8) [*:0]const u8 { return s.ptr; }
```

### ARM Discipline

```zig
// Use correct barriers after hardware register writes
mmio.write(reg, value);
barriers.dsb();  // Ensure write completes
barriers.isb();  // Synchronize instruction stream

// No hardcoded addresses - use DTB parsing or linker symbols
const uart_base = dtb.getNodeProperty("uart", "reg") orelse
    @extern([]const u8, .{ .name = "__uart_base" });
```

### Comptime & Generics

```zig
// Use comptime for zero-cost abstractions
fn Register(comptime offset: usize) type {
    return struct {
        pub fn read() u32 {
            return @as(*volatile u32, @ptrFromInt(base + offset)).*;
        }
        pub fn write(value: u32) void {
            @as(*volatile u32, @ptrFromInt(base + offset)).* = value;
        }
    };
}

// Prefer comptime assertions over runtime checks
comptime {
    if (@sizeOf(PageTable) != 4096) {
        @compileError("PageTable must be exactly 4KB");
    }
}
```

### Inline Assembly (ARM64)

```zig
// Use asm for system register access
fn readSCTLR() u64 {
    return asm volatile ("mrs %[ret], sctlr_el1"
        : [ret] "=r" (-> u64)
    );
}

// Memory barriers
fn dsb() void {
    asm volatile ("dsb sy" ::: "memory");
}
```

### Packed Structs for Hardware

```zig
// Use packed structs for MMIO registers
const UartFlags = packed struct(u32) {
    cts: bool,
    dsr: bool,
    dcd: bool,
    busy: bool,
    rxfe: bool,  // RX FIFO empty
    txff: bool,  // TX FIFO full
    rxff: bool,  // RX FIFO full
    txfe: bool,  // TX FIFO empty
    _reserved: u24 = 0,
};

// Read as structured data
const flags: UartFlags = @bitCast(mmio.read(UART_FR));
if (flags.txff) {
    // TX FIFO is full, wait
}
```

---

## Testing Philosophy: Diagnostics, Not Verdicts

**Tests are diagnostic tools, not success criteria.** A passing test suite does not mean the kernel is good. A failing test does not mean the code is wrong.

**When a test fails, ask three questions in order:**
1. Is the test itself correct and valuable?
2. Does the test align with our current design vision?
3. Is the code actually broken?

Only if all three answers are "yes" should you fix the code.

**Why this matters:**
- Tests encode assumptions. Assumptions can be wrong or outdated.
- Changing code to pass a bad test makes the codebase worse, not better.
- A research OS explores new territory - legacy testing assumptions don't always apply.

**What tests ARE good for:**
- **Regression detection**: Did a scheduler refactor break ICC? Did memory changes break container isolation?
- **Sanity checks**: Does boot complete? Do syscalls return? Does the happy path work?
- **Behavior documentation**: Tests show what the code currently does, not necessarily what it should do.

**What tests are NOT:**
- A definition of correctness
- A measure of kernel quality
- Something to "make pass" at all costs
- A specification to code against

**The real success metric**: Does the kernel further our vision of a container-native OS that developers and LLMs love to work with?

---

## Testing Guidelines

### Test Commands

```bash
# Single variant with timeout (quick iteration)
zig build run-timeout -Dtimeout=10000

# All variants with timeout (CI/regression testing)
zig run scripts/test_all.zig -- --timeout=10000

# Verbose mode for debugging boot issues
zig run scripts/test_all.zig -- --timeout=10000 --verbose
```

### Filtering Test Output (Windows)

QEMU produces verbose output. Filter to find relevant information:

```powershell
# Run and filter for specific patterns
zig build run-timeout -Dtimeout=10000 2>&1 | Select-String -Pattern "ERROR|PANIC|OK"

# Filter for smoke tokens only
zig build run-timeout -Dtimeout=10000 2>&1 | Select-String -Pattern "\[.*_OK\]"

# Filter for a specific subsystem
zig build run-timeout -Dtimeout=10000 2>&1 | Select-String -Pattern "ASID|TLB|container"

# Save full output and filter separately
zig build run-timeout -Dtimeout=10000 2>&1 | Out-File qemu-output.txt
Select-String -Path qemu-output.txt -Pattern "fault|abort" -Context 5,5
```

### Test Patterns

- Use `user/programs/` for integration test containers

### Smoke Token System

Tests emit tokens like `[BOOT_OK]`, `[SCHED_OK]`, `[NET_OK]`. The `test_all.zig` script captures and validates these across variants.

| Token | Meaning |
|-------|---------|
| `[BOOT_OK]` | Kernel booted successfully (required) |
| `[SCHED_OK]` | Scheduler initialized (required) |
| `[MMU_OK]` | MMU enabled correctly |
| `[NET_OK]` | Network stack initialized |
| `[HTTP_OK]` | HTTP test passed |
| `[BUILD_OK]` | Build system test passed |

Regression = missing previously-seen tokens across variants.

---

## Common Development Workflows

### Adding a New Syscall

1. **Define in table**: Edit `src/kernel/syscalls/table.zig`
   - Add SyscallSpec to `table` array
   - Choose number in category range:
     - Core: 100-119
     - ICC: 120-139
     - Memory: 140-159
     - Device: 160-179
     - Block: 180-199
     - Network: 200-219
     - Filesystem: 220-239
   - Define args (max 6 per ARM64 ABI), return type, error set, capabilities

2. **Implement handler**: Add to appropriate file
   - Core: `src/kernel/syscalls/core.zig`
   - ICC: `src/kernel/icc/handlers.zig`
   - Device: `src/kernel/syscalls/device.zig`

3. **Wire dispatch**: Update `src/kernel/syscalls/dispatch.zig`
   - Add case to category-specific dispatch function

4. **Regenerate and verify**:
   ```bash
   zig build gen-lib && zig build gen-docs
   zig run scripts/test_all.zig -- --timeout=10000
   ```

### Adding a New Container Type

1. Edit `src/kernel/container/table.zig`
   - Add ContainerTypeSpec to `type_table`
   - Define capabilities, exception level, page table template

2. Run `zig build gen-docs` to update documentation

3. Add spawn logic if needed in `user/services/lamina.zig`

### Stack Budget Reference

| Component | Size | Notes |
|-----------|------|-------|
| Exception frame | 288 bytes | Lazy SIMD (was 800) |
| Kernel stack | 32KB | Per container (was 128KB) |
| Nested syscall margin | 3x | Budget ~928 bytes worst-case |

---

## Debugging Tools

```bash
# Address to symbol resolution
llvm-objdump -d zig-out/bin/laminae-kernel-debug | grep <address>
# OR
llvm-symbolizer --obj=zig-out/bin/laminae-kernel-debug <address>

# Memory inspection
llvm-objdump -s --start-address=<start> --stop-address=<stop> zig-out/bin/laminae-kernel-debug
```

---

## Code Search Tools

### ripgrep (rg) - Fast Text Search

Use when searching for literal strings or regex patterns:

```bash
rg "unreachable" -t zig           # Find all unreachable statements
rg "TODO|FIXME" -t zig            # Find todos
rg "pub fn" src/                  # Find public functions
rg -l "Container" -t zig          # List files containing pattern
rg -n "inline fn" -t zig          # Show line numbers
```

### ast-grep - Structural Code Search

Use when you need syntax-aware matching:

```bash
ast-grep run -l zig -p '$X.?'              # Find all optional unwraps
ast-grep run -l zig -p '@panic($$$)'       # Find all panic calls
ast-grep run -l zig -p 'fn $NAME() void'   # Find functions returning void
```

**When to use which:**
- **ripgrep**: Quick searches, TODOs, config values, recon
- **ast-grep**: Refactoring patterns, finding anti-patterns, policy checks

---

## Development Scripts

Scripts live in `scripts/`. Run with `zig run scripts/<name>.zig -- <args>`.

Key scripts: `test_all.zig` (multi-variant testing), `gen_lib.zig` (syscall wrapper generation), `gen_docs.zig` (API docs).

---

## Bug Severity (Zig Freestanding)

### Critical - Must Fix Immediately

- `.?` on null (panics, crashes kernel)
- `unreachable` reached at runtime
- Index out of bounds on slices/arrays
- Integer overflow in release builds (undefined behavior)
- Use-after-free or double-free
- Unaligned pointer access on ARM (data abort)
- Missing memory barriers after MMIO writes (silent corruption)
- Exception faults (data abort, instruction abort, undefined instruction)
- Stack overflow (no guard pages in freestanding)

### Important - Fix Before Merge

- Missing error handling (`try` without proper catch/return)
- `catch unreachable` without justification comment
- ASID/TLB invalidation omitted after page table changes
- Volatile reads/writes missing for MMIO
- Incorrect packed struct layout for hardware registers
- Ignoring return values from functions returning `!T`
- Memory leaks in long-running code paths
- Race conditions in interrupt handlers

### Contextual - Address When Convenient

- TODO/FIXME comments
- Unused imports or variables
- Suboptimal comptime usage (could be comptime but isn't)
- Redundant code that could use generics
- Missing `inline` on hot path functions
- Excessive debug output left in code

---

## Development Philosophy

**Make it work, make it right, make it fast** - in that order. Use extensive `debugz()` output during development, strip aggressively once working.

**Closed-Loop Testing**: Build testable systems independently. Use `zig build run-timeout -Dtimeout=5000` for verification.

**No Hardcoded Values**: Use DTB parsing or linker symbols, not magic addresses.

---

## Linux Inspiration vs. Recreation

**We are not recreating Linux** - but we also do not reinvent the wheel. Take inspiration from proven patterns.

### What We Borrow (Standard ARM64 Patterns)

These are industry-standard patterns that Linux implements well. Use them:

| Pattern | Linux Reference | Our Usage |
|---------|-----------------|-----------|
| ARM64 ABI | syscall in x8, args x0-x5 | Same - it's the ARM spec |
| Safe copy | `copy_to_user` fault recovery | `src/kernel/syscalls/uaccess.zig` |
| Memory barriers | dmb/dsb/isb sequences | `src/arch/aarch64/common/barriers.zig` |
| MAIR_EL1 | Memory attribute config | Standard ARM setup |
| ASID | TLB isolation | Per-container isolation |
| GICv2 | Interrupt controller | `src/kernel/drivers/gic/v2.zig` |
| PL011 UART | Serial driver | `src/kernel/drivers/uart/pl011.zig` |
| DTB parsing | Flattened Device Tree | `src/kernel/fdt.zig` |
| Page tables | 4-level, 4KB granules | `src/kernel/container/page_tables.zig` |

### Where We Diverge (Container-Native Design)

These are deliberate architectural differences:

| Linux Approach | Laminae Approach | Rationale |
|----------------|------------------|-----------|
| Process model (PIDs) | Container model (CIDs) | Containers are first-class citizens |
| Copy-on-Write | Deep copy page tables | Simplicity over optimization |
| POSIX compatibility | Clean-slate syscalls | No 30 years of legacy |
| Per-process ASID | Per-container ASID | Direct hardware isolation mapping |
| Monolithic + modules | Microkernel-ish | Drivers in EL0 containers |

### When to Look at Linux

- **DO**: Reference `arch/arm64/` for ARM system register setup, barrier sequences, exception handling patterns
- **DO**: Study driver register definitions and initialization sequences
- **DO**: Learn from their TLB invalidation and cache management code
- **DON'T**: Copy their process/thread model, scheduler complexity, or POSIX shims
- **DON'T**: Add compatibility layers to match Linux behavior

**The goal**: A modern kernel that a Linux ARM64 developer would recognize at the hardware level, but find refreshingly simple at the OS level.

---

we love you, Claude! do your best today