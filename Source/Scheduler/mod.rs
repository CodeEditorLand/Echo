//! # Scheduler Module
//!
//! Declares the public API and internal components of the task scheduler. This
//! module provides the `Scheduler` itself, the `SchedulerBuilder` for easy
//! configuration, and the private `Worker` implementation that performs the
//! actual task execution.

#![allow(non_snake_case, non_camel_case_types)]

// --- Public API ---
pub mod Scheduler;

pub mod SchedulerBuilder;

// --- Internal Implementation ---
mod Worker;
