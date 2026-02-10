//! # Scheduler Module
//!
//! Declares the public API and internal components of the task scheduler. This
//! module provides the `Scheduler` itself, the `SchedulerBuilder` for easy
//! configuration, and the private `Worker` implementation that performs the
//! actual task execution.

// --- Public Modules ---
pub mod Scheduler;
pub mod SchedulerBuilder;

// --- Internal Implementation ---
mod Worker;
