use std::sync::{
	Arc,
	atomic::{AtomicBool, Ordering},
};

use log::trace;
use tokio::time::{Duration, sleep};

use crate::{Queue::StealingQueue::Context, Task::Task::Struct as TaskStruct};

pub struct Worker {
	Context:Context<TaskStruct>,

	Running:Arc<AtomicBool>,
}

impl Worker {
	pub fn New(Context:Context<TaskStruct>, IsRunning:Arc<AtomicBool>) -> Self { Self { Context, Running:IsRunning } }

	/// The main execution loop for the worker.
	pub async fn Run(&self) {
		trace!("[Worker {}] Starting execution loop.", self.Context.Identifier);

		while self.Running.load(Ordering::Relaxed) {
			let TaskOption = self.Context.NextTask();

			if let Some(Task) = TaskOption {
				trace!(
					"[Worker {}] Found task with priority {:?}. Executing.",
					self.Context.Identifier, Task.Priority
				);

				Task.Future.await;
			} else {
				sleep(Duration::from_millis(1)).await;
			}
		}

		trace!("[Worker {}] Execution loop finished.", self.Context.Identifier);
	}
}
