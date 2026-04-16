# Changelog

All notable changes to the Echo element are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.6.0] — 2026 Q2

### Changed

- Simplified rustdocflags configuration formatting
- Updated Documentation/Rust submodule to latest

### Added

- Comprehensive getting-started guide in README
- Benefit-focused crate-level rustdoc in Library.rs
- See Also section linking to architecture overview and related Elements

## [0.5.0] — 2026 Q1

### Fixed

- Disabled bugged examples that caused CI failures

### Changed

- Updated Cloudflare development dependencies (Wrangler, Miniflare,
  Workers Types)
- Upgraded @playform/build from 0.2.5 to 0.3.0
- Updated dependencies

## [0.4.0] — 2025 Q4

### Changed

- Updated dependencies (Wrangler 4.50 through 4.56, Miniflare, Workers Types)
- Upgraded CI actions (actions/cache 5.0, actions/checkout 6.x,
  actions/upload-artifact 6.0, actions/setup-node 6.x)

## [0.3.0] — 2025 Q3

### Added

- TypeScript declaration generation via tsconfig update
- Deep Dive architecture document for the scheduler
- Performance optimization roadmap and contribution guide

### Changed

- Decoupled queue from scheduler and optimized work-stealing algorithm
- Exposed scheduler APIs and optimized worker task polling
- Improved code clarity, documentation, and API consistency
- Reorganized Cargo.toml metadata and updated build artifacts
- Removed unused dependencies and development features
- Updated Cloudflare Workers dependencies

## [0.2.0] — 2025 Q2

### Changed

- Optimized release build configuration and resolved dependency version
  conflicts
- Updated @cloudflare/workers-types dependency
- Removed obsolete build artifacts
- Updated .gitignore for cleaner tracking
- Relicensed project under CC0 1.0 Universal, then migrated to Land Public
  License v1.0

## [0.1.0] — 2025 Q1

### Changed

- Updated dependencies

## [0.0.1] — 2024 Q3

### Added

- Initial Rust work-stealing task scheduler implementation
- Cloudflare Workers integration with Wrangler and Miniflare
- TypeScript worker bindings via @playform/build
- CI/CD workflows with GitHub Actions (cache, upload-artifact, setup-node)
- Dependabot configuration for automated dependency updates
