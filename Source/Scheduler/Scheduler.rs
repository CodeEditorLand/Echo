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

/// @module Scheduler
/// @description Defines the main `Scheduler` that manages the worker pool, task
/// queue, and task execution lifecycle.
use super::Worker::Worker;
use crate::{
	Queue::StealingQueue::StealingQueue,
	Scheduler::SchedulerBuilder::Concurrency,
	Task::{Priority::Enum, Task::Struct},
};

/// Manages a pool of worker threads and a work-stealing queue to execute tasks
/// efficiently. This struct is the public-facing API of the Echo scheduler.
pub struct Scheduler {
	/// The underlying work-stealing queue shared by all workers.
	Queue:Arc<StealingQueue>,
	/// Handles to the spawned worker threads, allowing for graceful shutdown.
	WorkerHandles:Vec<JoinHandle<()>>,
	/// An atomic flag to signal workers to shut down.
	IsRunning:Arc<AtomicBool>,
}

impl Scheduler {
	/// Creates and starts a new scheduler with a given configuration.
	/// This is a crate-private function, intended to be called only by the
	/// `SchedulerBuilder`.
	///
	/// @param number_of_workers - The number of worker threads to spawn.
	/// @param _queue_configs - Configuration for named queues with concurrency
	/// limits (future use).
	pub(crate) fn Start(number_of_workers:usize, _queue_configs:HashMap<String, Concurrency>) -> Self {
		info!("[Scheduler] Starting scheduler with {} worker threads.", number_of_workers);
		let IsRunning = Arc::new(AtomicBool::new(true));
		let Queue = Arc::new(StealingQueue::New(number_of_workers));

		let mut WorkerHandles = Vec::with_capacity(number_of_workers);

		for WorkerIdentifier in 0..number_of_workers {
			let CloneQueue = Queue.clone();
			let CloneIsRunning = IsRunning.clone();

			let WorkerHandle = tokio::spawn(async move {
				let WorkerInstance = Worker::New(WorkerIdentifier, CloneQueue, CloneIsRunning);
				WorkerInstance.Run().await;
			});

			WorkerHandles.push(WorkerHandle);
		}

		Self { Queue, WorkerHandles, IsRunning }
	}

	/// Submits a new task (as a `Future`) to the scheduler's global queue.
	/// The task will be picked up by the next available worker.
	///
	/// @param future_instance - The async block or function to execute.
	/// @param task_priority - The priority of the task.
	pub fn submit<F>(&self, future_instance:F, task_priority:Enum)
	where
		F: Future<Output = ()> + Send + 'static, {
		let new_task = Struct::New(future_instance, task_priority);
		self.Queue.Push(new_task);
	}

	/// Asynchronously shuts down the scheduler.
	///
	/// This signals all worker threads to stop their loops and then waits for
	/// them to complete their current tasks and exit gracefully.
	pub async fn ShutDown(&mut self) {
		if !self.IsRunning.swap(false, Ordering::Relaxed) {
			info!("[Scheduler] ShutDown already initiated.");
			return;
		}

		info!("[Scheduler] Shutting down worker threads...");
		for handle in self.WorkerHandles.drain(..) {
			if let Err(e) = handle.await {
				error!("[Scheduler] Error joining worker task during shutdown: {}", e);
			}
		}
		info!("[Scheduler] All workers shut down successfully.");
	}
}

impl Drop for Scheduler {
	/// Ensures that the scheduler is shut down when it goes out of scope,
	/// preventing orphaned worker threads.
	fn drop(&mut self) {
		if self.IsRunning.load(Ordering::Relaxed) {
			// If the scheduler is dropped without an explicit async shutdown,
			// we must signal the workers to stop. We cannot await the handles
			// here, but the threads will eventually terminate.
			warn!("[Scheduler] Scheduler dropped without explicit shutdown. Signaling workers to stop.");
			self.IsRunning.store(false, Ordering::Relaxed);
		}
	}
}
