# Changelog

All notable changes to Echo (Task Scheduler) are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/).

## [v2.0] - Q1 2026: Editor Launch Sprint

### Changed

- Documentation and config refinement
- StealingQueue.rs comment/doc updates
- Architecture mature and stable; no API changes

## [v1.3] - Q4 2025: Dependency Maintenance

### Changed

- Dependency updates maintained; no source changes

## [v1.2] - Q3 2025: Full Stack Integration

### Changed

- Build artifacts (libEcho.rlib) removed from version control
- Documentation polished: README (+46/-20), Knowledge.dot graph updated
  (+71/-51), example files reformatted
- CODE_OF_CONDUCT email updated
- Architecture stable; no code changes

## [v1.1] - Q2 2025: Architecture Buildout

**June 10, 2025: complete Sequence → Scheduler architecture flip.**

### Added (via Source2/ staging, then promoted)

- `Source/Queue/StealingQueue.rs` - work-stealing queue using
  `crossbeam-deque` with random peer iteration for reduced contention
- `Source/Queue/mod.rs`
- `Source/Scheduler/Scheduler.rs` - scheduler orchestration with public
  `Create` method
- `Source/Scheduler/SchedulerBuilder.rs` - fluent builder: `WithWorkerCount()`,
  `WithQueue()`
- `Source/Scheduler/Worker.rs` - prioritized local polling + system stealing
  strategies (99 insertions major enhancement)
- `Source/Scheduler/mod.rs`
- `Source/Task/Priority.rs` - High/Low priority enum for UI responsiveness vs
  background indexing
- `Source/Task/Task.rs` - task definition with priority support
- `Source/Task/mod.rs`
- `Source/Library.rs` - crate rustdoc + module exports

### Changed (June 10-19: 3 refinement commits)

- Worker polling refactored: prioritized local polling + system stealing
  (47 insertions, 27 deletions)
- Queue decoupled from scheduler: random peer iteration for efficient task
  distribution (66 insertions)
- API renamed for fluency: `Count` → `WithWorkerCount`, `Queue` →
  `WithQueue`, `Handle` → `WorkerHandles`, `Running` → `IsRunning`
- Code clarity pass: 264 insertions, 335 deletions (net -71 lines simplified)
- `docs/Deep Dive.md` rewritten: 94 insertions, 51 deletions
- README expanded: 59+42+24 insertions across 3 commits

### Removed

- Entire Sequence architecture: `Source/Struct/Sequence/*`, `Source/Trait/`,
  `Source/Type/`, `Source/Enum/`

## [v1.0] - Q1 2025: Integration Phase

### Changed

- Full PascalCase enforcement across 18 files (150 insertions, 164 deletions):
  spacing `Argument:Vec` → `Argument: Vec`, field init `Name:"Read"` →
  `Name: "Read"`
- Affected: all Example/ files, Source/Struct/Sequence/ subtree,
  Source/Trait/Sequence/, Source/Type/Sequence/, build.rs
- Wrangler v3 → v4 migration (Cloudflare Workers)

## [v0.2] - Q4 2024: Architecture Solidification

### Added

- LICENSE file (109 lines)

### Changed

- PascalCase formatting prep across Example files (23 insertions, 23
  deletions): Read.rs, Write.rs, Sequence.rs, Tauri.rs, WorkSteal.rs
- Cargo.toml metadata reorganization

## [v0.1] - Q3 2024: Rapid Development

### Added

- `Target/` directory with transpiled JS: Function/Response.js,
  Interface/{Data,Environment,Message,Response,Worker}.js,
  Variable/Worker.js
- Sequence-based task coordination architecture:
  - `Source/Struct/Sequence/` - Action, Life, Plan, Production, Signal, Vector
  - `Source/Trait/Sequence/` - Action, Site
  - `Source/Type/Sequence/` - Action/Cycle
- Example programs: Common/Read.rs, Common/Write.rs, Sequence.rs, Tauri.rs,
  WorkSteal.rs

### Removed

- `.github/workflows/Cloudflare.yml` (61 lines) - Workers deployment abandoned

### Dependencies (First Release)

- crossbeam-deque (work-stealing core), tokio, rand, num_cpus, log, serde
