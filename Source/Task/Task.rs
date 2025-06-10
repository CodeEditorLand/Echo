use std::{future::Future, pin::Pin};

/// @module Task
/// @description Defines the `Task` struct, which is the internal unit of work
/// for the Echo scheduler.
use super::Priority::Enum;
use crate::Queue::StealingQueue::{Prioritized, Priority};

/// A type alias for a boxed, send-able, pinned future that returns no value.
/// This is the standard way to handle dynamic, async operations in Rust.
type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Represents a single, schedulable unit of work.
///
/// It encapsulates an asynchronous operation (`Future`) along with metadata,
///
/// such as its `Priority`, that the scheduler can use to determine execution
/// order.
pub struct Struct {
	/// The asynchronous operation to be executed by a worker.
	pub Future:BoxedFuture,

	/// The priority level of this task, used by the scheduler's queue.
	pub Priority:Enum,
}

impl Struct {
	/// Creates a new `Task` from a given future and priority level.
	///
	/// @param FutureInstance - Any `Future` that is `Send` and has a `'static`
	///   lifetime. The future is automatically boxed and pinned.
	/// @param PriorityValue - The `Priority` of the task.
	pub fn New<F>(FutureInstance:F, PriorityValue:Enum) -> Self
	where
		F: Future<Output = ()> + Send + 'static, {
		Self { Future:Box::pin(FutureInstance), Priority:PriorityValue }
	}
}

impl Prioritized for Struct {
	type P = Priority;

	fn GetPriority(&self) -> Self::P {
		match self.Priority {
			Enum::High => Priority::High,

			Enum::Normal => Priority::Normal,

			Enum::Low => Priority::Low,
		}
	}
}
