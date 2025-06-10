//! Defines the execution priority level of a task.

#![allow(non_snake_case, non_camel_case_types)]

/// Represents the priority of a task to be executed.
///
/// This enumeration allows the scheduler to ensure that high-priority,
/// user-facing operations are executed before long-running background tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
	/// For tasks that directly impact perceived performance.
	High,
	/// The default priority for most standard operations.
	Normal,
	/// For background tasks that are not time-sensitive.
	Low,
}
