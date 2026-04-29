# Changelog - Echo

Echo is our high-performance work-stealing task scheduler - the Rust
crate Mountain pulls in for fan-out workloads where a single tokio
runtime would either contend or under-saturate. This file records what
we built in our voice, version by version. Format adapted from
[Keep a Changelog](https://keepachangelog.com/).

## [v2.1] - Full Workbench Lift (April 2026)

We did a documentation and surface-polish pass on Echo while the rest
of the workbench-lift cycle was happening around it.

### Added

- **Comprehensive CHANGELOG history** rebuilt from the commit record
  on 2026-04-16 / 2026-04-17 (`bb47864`, `6e05393`).
- **Crate-level rustdoc** rewritten benefit-first in `Library.rs`
  (`416b0f7`, 2026-04-04) so `cargo doc` lands a meaningful landing
  page for the scheduler.
- **README expansion** with a comprehensive getting-started guide
  (`7435fd3`, 2026-04-05) and a problem-first opening pass
  (`9b0b68b`, 2026-04-04). Architecture overview links added under
  See Also (`bfb451c`).

### Changed

- **`build.rs` cleanup**: imports moved to the top, `allow` attribute
  added (`89ce4ca`, 2026-04-20) so a clean `cargo build` produces no
  warnings.
- **CHANGELOG formatting standardised** (`4c5f86f`, 2026-04-21).

## [v2.0] - Editor Launch (Q1 2026)

We carried the scheduler through the editor-launch cycle without
breaking the Mountain integration. The workbench-fix commits in this
window (`aa2b4f9`, `416e963`, both labelled `"fix(workbench)"`) were
where Echo's call sites in Mountain stabilised against the new
workbench boot order.

### Changed

- **Workbench fixes** (2026-02-13, 2026-02-14) - the consumer side in
  Mountain learned to schedule its fan-out work through Echo cleanly
  during the new bundled-electron boot path.

## [v1.3] - Dependency Maintenance (Q4 2025)

`@cloudflare/workers-types`, `wrangler`, and `miniflare` rolled
forward continuously. No source changes - the crate stayed stable
through the back half of the year.

## [v1.2] - Full-Stack Integration (Q3 2025)

`tsconfig` updated for declaration generation (`2a22ebc`, 2025-07-03).
Documentation example formatting fixed (`6ed8a51`, 2025-07-06).
Routine `@cloudflare/workers-types` and `miniflare` rolls.

## [v1.1] - Architecture Buildout (Scheduler Pivot, June 2025)

The pivot cycle. Echo went from a hybrid scaffold to a real Rust
work-stealing scheduler with a published API surface.

### Added

- **Scheduler / Queue / Task module split** (`c92ea95`, 2025-06-10).
  First substantive Rust source landed:
  - `Source/Scheduler/{Scheduler,SchedulerBuilder,Worker}.rs`
  - `Source/Queue/StealingQueue.rs`
  - `Source/Task/{Task,Priority}.rs`
  - `Source/Library.rs` as the crate root.
- **PascalCase module conversion** in the same window - the initial
  lowercase `scheduler/`, `queue/`, `task/` directories from
  `64fca95` were re-spelled to `Scheduler/`, `Queue/`, `Task/` to
  match the project rule.
- **Deep Dive document** outlining scheduler architecture (`eab1531`,
  2025-06-08).
- **README rewrite** aligned with Land architecture and technical
  details (`9841b98`, 2025-06-08).
- **Performance optimisation roadmap** + contribution guide
  (`0f25c85`, `d88c3e7`, 2025-06-10).
- **Mountain/Scheduler core module documentation** (`b555aa6`,
  2025-06-10) - the consumer-side docs landed alongside the producer.

### Changed

- **Scheduler decoupled from queue** (`6d265a0`, 2025-06-10). Made
  the work-stealing path a first-class concern instead of an
  implementation detail buried in scheduler init.
- **Scheduler APIs exposed**, worker task polling optimised
  (`64fca95`, 2025-06-10).
- **Code clarity, documentation, and API consistency** sweep
  (`a16d402`, 2025-06-19).
- **`Cargo.toml` metadata reorganised**, build artefact updated
  (`fb9f4d1`, 2025-06-20).
- **Unused dependencies and dev features removed** (`d8f7788`,
  2025-06-11).
- **Release-build configuration optimised**, dependency-version
  conflict resolved (`7158ac9`, 2025-05-29).

## [v1.0] - Integration Phase (May 2025)

We sorted out licensing and the build before the June scheduler
buildout.

### Changed

- **Re-licensed to CC0 1.0 Universal** (`f4b30ec`, 2025-05-22).
- **Migrated from CC0 to Land Public License v1.0** (`32589c4`,
  2025-05-10) - the licence we converged on across the fleet.
- **Documentation URLs normalised to lowercase** (`0dbaf96`,
  2025-05-20).
- **`@playform/build` bumped to 0.2.4** (`0c8980f`, 2025-05-07).

## [v0.2] - Architecture Solidification (Q4 2024)

The crate sat in maintenance through the back half of 2024 while the
fleet's Rust-side architecture firmed up around it. Routine
`@cloudflare/workers-types` and tooling rolls; **no substantive
source changes** in this window beyond initial type sketches like
`Source/Type/Sequence/Action/StartEnd.rs` (`0b0d7fa`, 2024-08-06).

## [v0.1] - Rapid Development (Q3 2024)

The Cloudflare Workers + Cargo.toml scaffold landed across July and
August 2024 as a long sequence of `squash!` commits and Dependabot
rolls. No published API yet - we were still feeling out whether the
scheduler should live as a Workers-adjacent crate or as a pure-Rust
sidecar.

## [v0.0] - Project Inception (July 2024)

First scaffold (`c978097`, 2024-07-02): Cloudflare Workers project
template, GitHub Actions workflows, Dependabot config, the empty
`Cargo.toml` shell. Echo existed as a placeholder for the work-
stealing scheduler we knew Mountain would need - the real
implementation arrived in v1.1.
