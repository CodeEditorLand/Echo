//! # Priority Enum
//!
//! Defines the execution priority level of a `Task` within the `Echo`
//! scheduler.

#![allow(non_snake_case, non_camel_case_types)]

/// Represents the priority of a task to be executed by the scheduler.
///
/// This enumeration allows the scheduler to ensure that high-priority,

/// user-facing operations (e.g., responding to UI input) are executed before
/// lower-priority, long-running background tasks (e.g., file indexing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
	/// For tasks that directly impact perceived performance and responsiveness.
	/// These are always executed first.
	High,

	/// The default priority for most standard operations that are not
	/// time-critical but should not be unnecessarily delayed.
	Normal,

	/// For background tasks that are not time-sensitive and can be deferred
	/// if higher-priority work is available.
	Low,
}
