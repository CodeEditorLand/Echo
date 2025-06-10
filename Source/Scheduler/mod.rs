//! Declares the public API and internal components of the task scheduler.
//!
//! This module provides the `Scheduler` itself, the `SchedulerBuilder` for
//! configuration, and the private `Worker` implementation.

#![allow(non_snake_case, non_camel_case_types)]

// --- Public API ---
pub mod Scheduler;

pub mod SchedulerBuilder;

// --- Internal Implementation ---
mod Worker;
