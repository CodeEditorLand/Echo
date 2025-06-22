//! # Queue Module
//!
//! This module encapsulates all logic for the generic, priority-aware,

//! work-stealing queue system, which is the foundational data structure of the
//! `Echo` scheduler.

#![allow(non_snake_case, non_camel_case_types)]

/// Provides the generic, priority-aware, work-stealing queue implementation.
pub mod StealingQueue;
