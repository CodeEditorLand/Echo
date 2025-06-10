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
///
/// Each worker runs in a dedicated asynchronous task, continuously polling for
/// work from its local queues or stealing from its peers.
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
	///
	/// This method consumes the `Worker` instance, taking full ownership of its
	/// fields. This design is critical for making the resulting `Future` safe
	/// to send across threads as required by `tokio::spawn`.
	pub async fn Run(self) {
		trace!("[Worker {}] Starting.", self.Context.Identifier);

		while self.Running.load(Ordering::Relaxed) {
			// Attempt to find a task from any available source.
			if let Some(Task) = self.Context.Next() {
				trace!("[Worker {}] Execute: {:?}.", self.Context.Identifier, Task.Priority);
				// Execute the task's future to completion.
				Task.Operation.await;
			} else {
				// If no work is found, yield to the OS to prevent busy-waiting
				// and conserve CPU resources.
				sleep(Duration::from_millis(1)).await;
			}
		}

		trace!("[Worker {}] Finished.", self.Context.Identifier);
	}
}
