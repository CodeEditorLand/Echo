//! Manages the pool of workers and the task queue system.

#![allow(non_snake_case, non_camel_case_types)]

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
/// tasks efficiently. This is the primary public-facing struct of the library.
pub struct Scheduler {
	/// The underlying work-stealing queue system used for task submission.
	Queue:StealingQueue<Task>,
	/// Handles to the spawned worker threads, used for graceful shutdown.
	Handle:Vec<JoinHandle<()>>,
	/// An atomic flag to signal all workers to shut down.
	Running:Arc<AtomicBool>,
}

impl Scheduler {
	/// Creates and starts a new scheduler with a given configuration.
	///
	/// This is a crate-private function, intended to be called only by the
	/// `SchedulerBuilder`'s `Build` method.
	pub(crate) fn Create(Count:usize, _Configuration:HashMap<String, Concurrency>) -> Self {
		info!("[Scheduler] Create with {} workers.", Count);
		let Running = Arc::new(AtomicBool::new(true));

		// Create the entire queue system and retrieve the contexts for each worker.
		let (Queue, Contexts) = StealingQueue::<Task>::Create(Count);

		let mut Handle = Vec::with_capacity(Count);
		// Spawn an asynchronous task for each worker.
		for Context in Contexts.into_iter() {
			let Running = Running.clone();
			Handle.push(tokio::spawn(async move {
				// Each task creates and runs a worker, consuming its context.
				Worker::Create(Context, Running).Run().await;
			}));
		}

		Self { Queue, Handle, Running }
	}

	/// Submits a new task to the scheduler's global queue.
	///
	/// The task will be picked up by the next available worker according to its
	/// priority and the work-stealing logic.
	pub fn Submit<F>(&self, Operation:F, Priority:Priority)
	where
		F: Future<Output = ()> + Send + 'static, {
		self.Queue.Submit(Task::Create(Operation, Priority));
	}

	/// Asynchronously shuts down the scheduler.
	///
	/// This method signals all worker threads to stop their loops and then
	/// waits for each one to complete its current task and exit gracefully.
	pub async fn Stop(&mut self) {
		if !self.Running.swap(false, Ordering::Relaxed) {
			info!("[Scheduler] Stop already initiated.");
			return;
		}

		info!("[Scheduler] Stopping worker threads...");
		for Handle in self.Handle.drain(..) {
			if let Err(Error) = Handle.await {
				error!("[Scheduler] Error joining worker: {}", Error);
			}
		}
		info!("[Scheduler] All workers stopped.");
	}
}

impl Drop for Scheduler {
	/// Ensures workers are signaled to stop if the `Scheduler` is dropped.
	///
	/// This prevents orphaned worker threads if the user forgets to call the
	/// explicit `Stop` method.
	fn drop(&mut self) {
		if self.Running.load(Ordering::Relaxed) {
			warn!("[Scheduler] Dropped without explicit stop. Signaling workers.");
			self.Running.store(false, Ordering::Relaxed);
		}
	}
}
