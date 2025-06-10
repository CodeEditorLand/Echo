//! A single execution thread in the scheduler's pool.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::{
	Arc,
	atomic::{AtomicBool, Ordering},
};

use log::trace;
use tokio::time::{Duration, sleep};

use crate::{Queue::StealingQueue::Context, Task::Task::Task};

/// Represents a worker that executes tasks from its assigned context.
pub struct Worker {
	/// The worker's execution context, which contains its private queues
	/// and a reference to the shared queue system.
	Context:Context<Task>,
	/// An atomic flag, shared by all workers, to signal a shutdown request.
	Running:Arc<AtomicBool>,
}

impl Worker {
	/// Creates a new `Worker` with its unique execution context and a reference
	// to the scheduler's running state.
	pub fn Create(Context:Context<Task>, Running:Arc<AtomicBool>) -> Self { Self { Context, Running } }

	/// The main execution loop for the worker.
	pub async fn Run(self) {
		trace!("[Worker {}] Starting.", self.Context.Identifier);

		while self.Running.load(Ordering::Relaxed) {
			// First, try to get a task from the local queues.
			let TaskOption = self.PopLocal();

			if let Some(Task) = TaskOption {
				trace!("[Worker {}] Execute Local: {:?}.", self.Context.Identifier, Task.Priority);
				Task.Operation.await;
				continue; // Immediately loop back to check for more local work.
			}

			// If no local work, try to steal from the system.
			// This attempts to get a batch and executes the first task.
			// The rest of the batch now populates the local queue for the next loops.
			let TaskOption = self.StealFromSystem();

			if let Some(Task) = TaskOption {
				trace!("[Worker {}] Execute Stolen: {:?}.", self.Context.Identifier, Task.Priority);
				Task.Operation.await;
			} else {
				// If there's truly no work anywhere, yield.
				sleep(Duration::from_millis(1)).await;
			}
		}

		trace!("[Worker {}] Finished.", self.Context.Identifier);
	}

	/// Attempts to pop a single task from the local deques, honoring priority.
	fn PopLocal(&self) -> Option<Task> {
		self.Context.Local.0.pop() // High
			.or_else(|| self.Context.Local.1.pop()) // Normal
			.or_else(|| self.Context.Local.2.pop()) // Low
	}

	/// Attempts to steal a batch of work from the system.
	///
	/// It steals from the highest-priority queue that has work, populating its
	/// own local queue and returning the first task immediately.
	fn StealFromSystem(&self) -> Option<Task> {
		self.Context.Steal(&self.Context.Share.Injector.0, &self.Context.Share.Stealer.0, &self.Context.Local.0) // Steal High
			.or_else(|| self.Context.Steal(&self.Context.Share.Injector.1, &self.Context.Share.Stealer.1, &self.Context.Local.1)) // Steal Normal
			.or_else(|| self.Context.Steal(&self.Context.Share.Injector.2, &self.Context.Share.Stealer.2, &self.Context.Local.2)) // Steal Low
	}
}
