//! # Scheduler
//!
//! Manages the pool of workers and the task queue system, serving as the main
//! public interface of the `Echo` library.

use std::{
	collections::HashMap,
	future::Future,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
};

use log::{error, info, warn};
use tokio::task::JoinHandle;

use super::{SchedulerBuilder::Concurrency, Worker::Worker};
use crate::{
	Queue::StealingQueue::StealingQueue,
	Task::{Priority::Priority, Task::Task},
};

/// Manages a pool of worker threads and a work-stealing queue to execute
/// tasks efficiently. The primary public-facing struct of the Echo library.
pub struct Scheduler {
	/// The underlying work-stealing queue system used for task submission.
	Queue:StealingQueue<Task>,

	/// Handles to the spawned worker threads, used for graceful shutdown.
	WorkerHandles:Vec<JoinHandle<()>>,

	/// An atomic flag to signal all workers to shut down.
	IsRunning:Arc<AtomicBool>,
}

impl Scheduler {
	/// Creates and starts a new scheduler with a given configuration.
	///
	/// Called only by the `SchedulerBuilder::Build` method.
	///
	/// ## Parameters
	///
	/// * `WorkerCount` — Number of worker threads to spawn.
	/// * `_Configuration` — Named-queue concurrency limits (reserved for future
	///   use).
	pub(crate) fn Create(WorkerCount:usize, _Configuration:HashMap<String, Concurrency>) -> Self {
		info!("[Scheduler] Creating scheduler with {} workers.", WorkerCount);

		let IsRunning = Arc::new(AtomicBool::new(true));

		// Create the entire queue system and retrieve the contexts for each worker.
		let (Queue, Contexts) = StealingQueue::<Task>::Create(WorkerCount);

		let mut WorkerHandles = Vec::with_capacity(WorkerCount);

		// Spawn an asynchronous task for each worker.
		for Context in Contexts.into_iter() {
			let IsRunning = IsRunning.clone();

			let WorkerHandle = tokio::spawn(async move {
				// Each task creates and runs a worker, consuming its context.
				Worker::Create(Context, IsRunning).Run().await;
			});

			WorkerHandles.push(WorkerHandle);
		}

		Self { Queue, WorkerHandles, IsRunning }
	}

	/// Submits a new task to the scheduler's global queue.
	///
	/// The task is picked up by the next available worker according to its
	/// priority and the work-stealing logic.
	///
	/// ## Parameters
	///
	/// * `Operation` — The future to execute.
	/// * `Priority` — The execution priority for this task.
	pub fn Submit<F>(&self, Operation:F, Priority:Priority)
	where
		F: Future<Output = ()> + Send + 'static, {
		self.Queue.Submit(Task::Create(Operation, Priority));
	}

	/// Asynchronously shuts down the scheduler.
	///
	/// Signals all worker threads to stop their loops and waits for each one to
	/// complete its current task and exit gracefully.
	///
	/// ## Errors
	///
	/// Logs an error if a worker handle produces a join error but does not
	/// propagate the error to the caller.
	pub async fn Stop(&mut self) {
		if !self.IsRunning.swap(false, Ordering::Relaxed) {
			info!("[Scheduler] Stop already initiated.");

			return;
		}

		info!("[Scheduler] Stopping worker threads...");

		for Handle in self.WorkerHandles.drain(..) {
			if let Err(Error) = Handle.await {
				error!("[Scheduler] Error joining worker handle: {}", Error);
			}
		}

		info!("[Scheduler] All workers stopped successfully.");
	}
}

impl Drop for Scheduler {
	/// Ensures workers are signaled to stop if the `Scheduler` is dropped
	/// without an explicit call to `Stop`.
	///
	/// Prevents orphaned worker threads if the user forgets to manage the
	/// scheduler's lifecycle properly.
	fn drop(&mut self) {
		if self.IsRunning.load(Ordering::Relaxed) {
			warn!("[Scheduler] Dropped without explicit stop. Signaling workers to terminate.");

			self.IsRunning.store(false, Ordering::Relaxed);
		}
	}
}
