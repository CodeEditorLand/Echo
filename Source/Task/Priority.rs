//! # Priority
//!
//! Defines the execution priority level of a `Task` within the Echo
//! scheduler.

/// Execution priority of a task in the Echo scheduler.
///
/// Allows the scheduler to ensure that high-priority, user-facing operations
/// (e.g., responding to UI input) are executed before lower-priority,
/// long-running background tasks (e.g., file indexing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
	/// High — Tasks that directly impact perceived performance and
	/// responsiveness. These are always executed first.
	High,

	/// Normal — Default priority for standard operations that are not
	/// time-critical but should not be unnecessarily delayed.
	Normal,

	/// Low — Background tasks that are not time-sensitive and can be deferred
	/// if higher-priority work is available.
	Low,
}
