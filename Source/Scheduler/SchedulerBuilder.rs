use std::collections::HashMap;

use log::warn;

use super::Scheduler::Scheduler;

#[derive(Debug, Clone, Copy)]
pub enum Concurrency {
	Limit(usize),

	Unlimited,
}

pub struct SchedulerBuilder {
	WorkerCount:usize,

	QueueConfiguration:HashMap<String, Concurrency>,
}

impl SchedulerBuilder {
	pub fn New() -> Self {
		let DefaultWorkerCount = num_cpus::get().max(2);

		Self { WorkerCount:DefaultWorkerCount, QueueConfiguration:HashMap::new() }
	}

	pub fn WithWorkerCount(mut self, WorkerCount:usize) -> Self {
		if WorkerCount == 0 {
			warn!("[SchedulerBuilder] Worker count of 0 is invalid. Defaulting to number of logical CPUs.");

			self.WorkerCount = num_cpus::get().max(2);
		} else {
			self.WorkerCount = WorkerCount;
		}

		self
	}

	pub fn WithQueue(mut self, QueueName:&str, ConcurrencyLimit:Concurrency) -> Self {
		self.QueueConfiguration.insert(QueueName.to_string(), ConcurrencyLimit);

		self
	}

	pub fn Build(self) -> Scheduler { Scheduler::Start(self.WorkerCount, self.QueueConfiguration) }
}

impl Default for SchedulerBuilder {
	fn default() -> Self { Self::New() }
}
