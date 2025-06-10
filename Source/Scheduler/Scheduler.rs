use std::{
	collections::HashMap,
	future::Future,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	time::Duration,
};

use log::{error, info, trace, warn};
use tokio::task::JoinHandle;

// CHANGED: Use the new reusable library.
use crate::Queue::StealingQueue::StealingQueue;
use crate::{
	Scheduler::SchedulerBuilder::Concurrency,
	Task::{Priority::Enum, Task::Struct},
};

/// Manages a pool of worker threads and a work-stealing queue to execute tasks
/// efficiently. This struct is the public-facing API of the Echo scheduler.
pub struct Scheduler {
	// CHANGED: The Scheduler now holds an instance of our generic queue.
	Queue:StealingQueue<Struct>,
	/// Handles to the spawned worker threads, allowing for graceful shutdown.
	WorkerHandles:Vec<JoinHandle<()>>,
	/// An atomic flag to signal workers to shut down.
	IsRunning:Arc<AtomicBool>,
}

impl Scheduler {
	pub(crate) fn Start(number_of_workers:usize, _queue_configs:HashMap<String, Concurrency>) -> Self {
		info!("[Scheduler] Starting scheduler with {} worker threads.", number_of_workers);
		let IsRunning = Arc::new(AtomicBool::new(true));

		// 1. Create the queue system. This is now a single, clean line.
		let (Queue, WorkerContexts) = StealingQueue::New(number_of_workers);

		let mut WorkerHandles = Vec::with_capacity(number_of_workers);

		// 2. Iterate over the contexts, giving one to each new thread.
		for Context in WorkerContexts.into_iter() {
			let CloneIsRunning = IsRunning.clone();

			let WorkerHandle = tokio::spawn(async move {
				// The worker logic is now simple and lives directly inside the spawned task.
				trace!("[Worker] Starting execution loop.");
				while CloneIsRunning.load(Ordering::Relaxed) {
					// Use the context to find the next task.
					if let Some(Task) = Context.NextTask() {
						trace!("[Worker] Found task with priority {:?}. Executing.", Task.Priority);
						Task.Future.await
					} else {
						tokio::time::sleep(Duration::from_millis(1)).await;
					}
				}
				trace!("[Worker] Execution loop finished.");
			});

			WorkerHandles.push(WorkerHandle);
		}

		Self { Queue, WorkerHandles, IsRunning }
	}

	pub fn submit<F>(&self, future_instance:F, task_priority:Enum)
	where
		F: Future<Output = ()> + Send + 'static, {
		let new_task = Struct::New(future_instance, task_priority);
		// 3. Use the generic queue's submit method.
		self.Queue.Submit(new_task);
	}

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
	fn drop(&mut self) {
		if self.IsRunning.load(Ordering::Relaxed) {
			warn!("[Scheduler] Scheduler dropped without explicit shutdown. Signaling workers to stop.");
			self.IsRunning.store(false, Ordering::Relaxed);
		}
	}
}
