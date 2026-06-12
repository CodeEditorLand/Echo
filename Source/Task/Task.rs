//! # Task
//!
//! Defines the structure of a schedulable task to be executed by an Echo
//! `Worker`.

use std::{future::Future, pin::Pin};

use crate::{
	Queue::StealingQueue::{Prioritized, Priority as QueuePriority},
	Task::Priority::Priority,
};

/// A boxed, pinned, send-safe asynchronous operation.
pub type Operation = Pin<Box<dyn Future<Output = ()> + Send>>;

/// A single schedulable unit of work for the Echo scheduler.
///
/// Encapsulates an asynchronous operation together with metadata — such as its
/// `Priority` — that the scheduler uses to determine execution order.
pub struct Task {
	/// Operation — The asynchronous operation to be executed by a worker
	/// thread.
	pub Operation:Operation,

	/// Priority — The execution priority level of this task.
	pub Priority:Priority,
}

impl Task {
	/// Creates a new `Task` from a given future and priority level.
	///
	/// ## Parameters
	///
	/// * `Operation` — The future to execute.
	/// * `Priority` — The execution priority for this task.
	pub fn Create<F>(Operation:F, Priority:Priority) -> Self
	where
		F: Future<Output = ()> + Send + 'static, {
		Self { Operation:Box::pin(Operation), Priority }
	}
}

/// Bridges the `Task` priority with the internal `StealingQueue` priority system.
///
/// Enables the queue system to place each task in the correct priority deque.
impl Prioritized for Task {
	/// The kind of priority used by the queue system.
	type Kind = QueuePriority;

	/// Translates this task's priority into the queue's generic priority enum,
	/// placing the task into the correct priority deque.
	fn Rank(&self) -> Self::Kind {
		match self.Priority {
			Priority::High => QueuePriority::High,

			Priority::Normal => QueuePriority::Normal,

			Priority::Low => QueuePriority::Low,
		}
	}
}
