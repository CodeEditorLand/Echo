//! # Task Struct
//!
//! Defines the structure of a schedulable task to be executed by a `Worker`.

#![allow(non_snake_case, non_camel_case_types)]

use std::{future::Future, pin::Pin};

use crate::{
	Queue::StealingQueue::{Prioritized, Priority as QueuePriority},
	Task::Priority::Priority,
};

/// Defines a dynamic, asynchronous operation that can be sent between threads.
/// This is a type alias for a boxed, pinned, send-safe future.
pub type Operation = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Represents a single, schedulable unit of work for the `Echo` scheduler.
///
/// This struct encapsulates an asynchronous operation along with metadata,

/// such as its `Priority`, that the scheduler uses to determine execution
/// order.
pub struct Task {
	/// The asynchronous operation to be executed by a worker thread.
	pub Operation:Operation,

	/// The priority level of this task.
	pub Priority:Priority,
}

impl Task {
	/// Creates a new `Task` from a given future and priority level.
	pub fn Create<F>(Operation:F, Priority:Priority) -> Self
	where
		F: Future<Output = ()> + Send + 'static, {
		Self { Operation:Box::pin(Operation), Priority }
	}
}

/// Implements the `Prioritized` trait required by the generic `StealingQueue`.
///
/// This implementation provides the bridge between the application-specific
/// `Task::Priority` and the generic `Queue::StealingQueue::Priority` used
/// internally by the queue system.
impl Prioritized for Task {
	/// The kind of priority used by the queue.
	type Kind = QueuePriority;

	/// Translates this task's specific priority into the queue's generic
	/// priority enum, allowing the queue to place it in the correct deque.
	fn Rank(&self) -> Self::Kind {
		match self.Priority {
			Priority::High => QueuePriority::High,

			Priority::Normal => QueuePriority::Normal,

			Priority::Low => QueuePriority::Low,
		}
	}
}
