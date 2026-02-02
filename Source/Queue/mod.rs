//! # Queue Module
//!
//! This module encapsulates all logic for the generic, priority-aware,
//! work-stealing queue system, which is the foundational data structure of the
//! `Echo` scheduler.

/// Provides the generic, priority-aware, work-stealing queue implementation.
pub mod StealingQueue;
