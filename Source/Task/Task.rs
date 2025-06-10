use std::{future::Future, pin::Pin};

/// @module Task
/// @description Defines the `Task` struct, which is the internal unit of work
/// for the Echo scheduler.
use super::Priority::Priority;

/// A type alias for a boxed, send-able, pinned future that returns no value.
/// This is the standard way to handle dynamic, async operations in Rust.
type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Represents a single, schedulable unit of work.
///
/// It encapsulates an asynchronous operation (`Future`) along with metadata,
/// such as its `Priority`, that the scheduler can use to determine execution
/// order.
pub struct Task {
	/// The asynchronous operation to be executed by a worker.
	pub Future:BoxedFuture,
	/// The priority level of this task, used by the scheduler's queue.
	pub Priority:Priority,
}

impl Task {
	/// Creates a new `Task` from a given future and priority level.
	///
	/// @param FutureInstance - Any `Future` that is `Send` and has a `'static`
	///   lifetime. The future is automatically boxed and pinned.
	/// @param PriorityValue - The `Priority` of the task.
	pub fn New<F>(FutureInstance:F, PriorityValue:Priority) -> Self
	where
		F: Future<Output = ()> + Send + 'static, {
		Self { Future:Box::pin(FutureInstance), Priority:PriorityValue }
	}
}
