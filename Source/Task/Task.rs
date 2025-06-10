//! Defines the structure of a task to be executed.

#![allow(non_snake_case, non_camel_case_types)]

use std::{future::Future, pin::Pin};

use crate::{
	Queue::StealingQueue::{Prioritized, Priority as QueuePriority},
	Task::Priority::Priority,
};

/// Defines a dynamic, asynchronous operation that can be sent between threads.
pub type Operation = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Represents a single, schedulable unit of work.
///
/// This struct encapsulates an asynchronous operation along with metadata,
/// such as its `Priority`, that the scheduler uses to determine execution
/// order.
pub struct Task {
	/// The asynchronous operation to be executed by a worker.
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

/// Implements the trait required by the generic `StealingQueue`.
///
/// This implementation provides the bridge between the application-specific
/// `Task::Priority` and the generic `Queue::StealingQueue::Priority`.
impl Prioritized for Task {
	/// The kind of priority used by the queue.
	type Kind = QueuePriority;

	/// Translates this task's specific priority into the queue's generic
	/// format.
	fn Rank(&self) -> Self::Kind {
		match self.Priority {
			Priority::High => QueuePriority::High,
			Priority::Normal => QueuePriority::Normal,
			Priority::Low => QueuePriority::Low,
		}
	}
}
