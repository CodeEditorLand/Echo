use std::sync::{
	Arc,
	atomic::{AtomicBool, Ordering},
};

use log::trace;
use tokio::time::{Duration, sleep};

/// @module Worker (Scheduler)
/// @description Defines the `Worker` struct, which represents a single
/// execution thread in the scheduler's pool. This is an internal component of
/// the scheduler.
use crate::queue::StealingQueue;

/// Represents a single worker thread that continuously polls the work-stealing
/// queue for tasks to execute.
pub(crate) struct Worker {
	Id:usize,
	Queue:Arc<StealingQueue>,
	IsRunning:Arc<AtomicBool>,
}

impl Worker {
	pub fn New(Id:usize, Queue:Arc<StealingQueue>, IsRunning:Arc<AtomicBool>) -> Self { Self { Id, Queue, IsRunning } }

	/// The main execution loop for the worker.
	///
	/// It continuously attempts to find a task from its local queue or by
	/// stealing from other workers. When a task is found, its encapsulated
	/// `Future` is awaited to completion.
	pub async fn Run(&self) {
		trace!("[Worker {}] Starting execution loop.", self.Id);
		while self.IsRunning.load(Ordering::Relaxed) {
			// Attempt to get a task from the shared queue.
			let TaskOption = self.Queue.StealForWorker(self.Id);

			if let Some(Task) = TaskOption {
				trace!("[Worker {}] Found task with priority {:?}. Executing.", self.Id, Task.Priority);
				// The future is executed simply by awaiting it.
				// Any panics within the future will be caught by tokio's task system
				// and can be handled when the JoinHandle is awaited on shutdown.
				Task.Future.await;
			} else {
				// If no work is found anywhere, yield the thread to the OS to prevent
				// a tight busy-loop from consuming 100% CPU.
				sleep(Duration::from_millis(1)).await;
			}
		}
		trace!("[Worker {}] Execution loop finished.", self.Id);
	}
}
