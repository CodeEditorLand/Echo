/// @module Priority
/// @description Defines the `Priority` enum for task scheduling within the Echo
/// system.

/// Represents the priority of a task to be executed by the scheduler.
/// This enum implements `Ord`, allowing tasks to be sorted by priority.
/// Schedulers and workers can use this to ensure that high-priority,
/// user-facing tasks are executed before long-running background tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Enum {
	/// For background tasks that are not time-sensitive, such as logging,
	/// telemetry, or non-critical file indexing.
	Low,

	/// The default priority for most standard operations.
	Normal,

	/// For tasks that directly impact perceived performance or are critical to
	/// responsiveness, such as handling user input, providing code completions,
	/// or responding to a "Go to Definition" request.
	High,
}
