// @module scheduler
// @description This module defines the core public API for the Echo task
// scheduler. It provides the `SchedulerBuilder` for configuration and the
// `Scheduler` itself for managing the worker pool and submitting tasks.
//

#![allow(non_snake_case, non_camel_case_types)]

// --- Sub-modules (Internal Implementation) ---
mod Scheduler;
mod SchedulerBuilder;
mod Worker;

// --- Public Re-exports ---

// The main scheduler struct that manages the worker pool and task execution.
// @see Scheduler
//
pub use self::Scheduler::Scheduler;
// The fluent builder for creating and configuring a `Scheduler` instance.
// This is the primary entry point for using the Echo library.
// @see SchedulerBuilder
pub use self::SchedulerBuilder::SchedulerBuilder;
