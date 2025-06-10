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

use super::Worker::Worker;
use crate::{
	Queue::StealingQueue::StealingQueue as StealingQueueStruct,
	Scheduler::SchedulerBuilder::Concurrency as ConcurrencyEnum,
	Task::{Priority::Enum as PriorityEnum, Task::Struct as TaskStruct},
};

pub struct Scheduler {
	Queue:StealingQueueStruct<TaskStruct>,

	Handle:Vec<JoinHandle<()>>,

	Running:Arc<AtomicBool>,
}

impl Scheduler {
	pub fn Start(number_of_workers:usize, _queue_configs:HashMap<String, ConcurrencyEnum>) -> Self {
		info!("[Scheduler] Starting scheduler with {} worker threads.", number_of_workers);

		let Running = Arc::new(AtomicBool::new(true));

		let (Queue, WorkerContexts) = StealingQueueStruct::<TaskStruct>::New(number_of_workers);

		let mut Handle = Vec::with_capacity(number_of_workers);

		for Context in WorkerContexts.into_iter() {
			let Running = Running.clone();

			Handle.push(tokio::spawn(async move {
				let WorkerInstance = Worker::New(Context, Running);

				WorkerInstance.Run().await;
			}));
		}

		Self { Queue, Handle, Running }
	}

	pub fn Submit<F>(&self, future_instance:F, task_priority:PriorityEnum)
	where
		F: Future<Output = ()> + Send + 'static, {
		self.Queue.Submit(TaskStruct::New(future_instance, task_priority));
	}

	pub async fn ShutDown(&mut self) {
		if !self.Running.swap(false, Ordering::Relaxed) {
			info!("[Scheduler] ShutDown already initiated.");

			return;
		}

		info!("[Scheduler] Shutting down worker threads...");

		for handle in self.Handle.drain(..) {
			if let Err(e) = handle.await {
				error!("[Scheduler] Error joining worker task during shutdown: {}", e);
			}
		}

		info!("[Scheduler] All workers shut down successfully.");
	}
}

impl Drop for Scheduler {
	fn drop(&mut self) {
		if self.Running.load(Ordering::Relaxed) {
			warn!("[Scheduler] Scheduler dropped without explicit shutdown. Signaling workers to stop.");

			self.Running.store(false, Ordering::Relaxed);
		}
	}
}
