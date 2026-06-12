//! # Queue Module
//!
//! Generic, priority-aware, work-stealing queue system that serves as the
//! foundational data structure of the `Echo` scheduler.

/// Provides the generic, priority-aware, work-stealing queue implementation.
pub mod StealingQueue;
