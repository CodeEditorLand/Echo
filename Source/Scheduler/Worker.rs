//! # Worker
//!
//! Represents a single execution thread in the scheduler's thread pool.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::{
	Arc,
	atomic::{AtomicBool, Ordering},
};

use log::trace;
use tokio::time::{Duration, sleep};

use crate::{Queue::StealingQueue::Context, Task::Task::Task};

/// Represents a worker that executes tasks from its assigned `Context`.
pub struct Worker {
	/// The worker's execution context, which contains its private deques and a
	/// reference to the shared queue system.
	Context:Context<Task>,
	/// An atomic flag, shared by all workers, to signal a shutdown request.
	IsRunning:Arc<AtomicBool>,
}

impl Worker {
	/// Creates a new `Worker` with its unique execution context and a reference
	/// to the scheduler's running state.
	pub fn Create(Context:Context<Task>, IsRunning:Arc<AtomicBool>) -> Self { Self { Context, IsRunning } }

	/// The main execution loop for the worker.
	///
	/// This loop continuously tries to find and execute tasks. It prioritizes
	/// its local queue and, if empty, attempts to steal work from other
	/// workers or the global queue. If no work is found anywhere, it yields
	/// briefly to avoid busy-waiting.
	pub async fn Run(self) {
		trace!("[Worker {}] Starting run loop.", self.Context.Identifier);

		while self.IsRunning.load(Ordering::Relaxed) {
			// First, try to get a task from the local deques.
			let TaskOption = self.PopLocal();

			if let Some(Task) = TaskOption {
				trace!(
					"[Worker {}] Executing local task with priority: {:?}.",
					self.Context.Identifier, Task.Priority
				);
				Task.Operation.await;
				continue;
			}

			// If no local work, try to steal from the system.
			let TaskOption = self.StealFromSystem();

			if let Some(Task) = TaskOption {
				trace!(
					"[Worker {}] Executing stolen task with priority: {:?}.",
					self.Context.Identifier, Task.Priority
				);
				Task.Operation.await;
			} else {
				// If there's truly no work anywhere, yield to the OS.
				sleep(Duration::from_millis(1)).await;
			}
		}

		trace!("[Worker {}] Run loop finished.", self.Context.Identifier);
	}

	/// Attempts to pop a single task from the local deques, honoring priority
	/// from high to low.
	fn PopLocal(&self) -> Option<Task> {
		self.Context
			.Local
			.0
			.pop()
			.or_else(|| self.Context.Local.1.pop())
			.or_else(|| self.Context.Local.2.pop())
	}

	/// Attempts to steal a batch of work from the system.
	///
	/// It steals from the highest-priority queue that has work, populating its
	/// own local queue and returning the first task immediately for execution.
	fn StealFromSystem(&self) -> Option<Task> {
		self.Context
			.Steal(
				&self.Context.Share.Injector.0,
				&self.Context.Share.Stealer.0,
				&self.Context.Local.0,
			)
			.or_else(|| {
				self.Context.Steal(
					&self.Context.Share.Injector.1,
					&self.Context.Share.Stealer.1,
					&self.Context.Local.1,
				)
			})
			.or_else(|| {
				self.Context.Steal(
					&self.Context.Share.Injector.2,
					&self.Context.Share.Stealer.2,
					&self.Context.Local.2,
				)
			})
	}
}
