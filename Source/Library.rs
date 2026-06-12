#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(
	non_snake_case,
	non_camel_case_types,
	non_upper_case_globals,
	dead_code,
	unused_imports,
	unused_variables,
	unused_assignments
)]

//! # Echo: Work-Stealing Task Scheduler
//!
//! Echo keeps every CPU core busy. It uses a lock-free work-stealing queue
//! (`crossbeam-deque`) so that idle threads automatically pick up tasks from
//! busy ones. No central bottleneck, no wasted cores.
//!
//! ## Why Echo Instead of `tokio::spawn`
//!
//! Tokio is great for I/O-bound work, but CPU-bound tasks (parsing, diffing,
//! indexing) block the executor. Echo provides:
//!
//! - **Priority levels**: UI-blocking tasks pre-empt background indexing
//! - **Work stealing**: Idle workers take from busy workers' queues
//! - **Structured shutdown**: Graceful drain with timeout
//!
//! ## Usage
//!
//! ```rust,ignore
//! use Echo::Scheduler::SchedulerBuilder;
//! use Echo::Task::Priority;
//!
//! let Scheduler = SchedulerBuilder::new().Workers(4).Build();
//! Scheduler.Submit(Priority::High, async { /* critical work */ });
//! Scheduler.Submit(Priority::Low, async { /* background indexing */ });
//! ```
//!
//! ## Modules
//!
//! - [`Scheduler`] — Builder and runtime for the worker pool
//! - [`Queue`] — Lock-free work-stealing deque wrapper
//! - [`Task`] — Task definition with priority levels
//!
//! ## Links
//!
//! - [Repository](https://github.com/CodeEditorLand/Echo)
//! - [Architecture](https://github.com/CodeEditorLand/Echo/blob/main/Documentation/GitHub/Architecture.md)

// --- Crate Modules ---
// Declares the main modules that constitute the library.
pub mod Queue;

pub mod Scheduler;

pub mod Task;
