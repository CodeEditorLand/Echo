// Defines the fluent builder for creating and configuring a `Scheduler`
// instance.

use std::collections::HashMap;

use log::warn;

use super::Scheduler;

/// An enum to define concurrency limits for named queues.
#[derive(Debug, Clone, Copy)]
pub enum Concurrency {
	/// Specifies a maximum number of concurrent tasks for a queue.
	Limit(usize),
	/// Allows an unlimited number of concurrent tasks for a queue.
	Unlimited,
}

/// A fluent builder for creating a `Scheduler`.
///
/// This pattern provides a clear and readable API for configuring complex
/// objects, such as a multi-queue, multi-threaded scheduler, before they are
/// constructed.
pub struct SchedulerBuilder {
	WorkerCount:usize,
	/// Stores the configuration for named queues.
	QueueConfiguration:HashMap<String, Concurrency>,
}

impl SchedulerBuilder {
	/// Creates a new `SchedulerBuilder` with default settings.
	///
	/// By default, the worker count is set to the number of logical CPUs on the
	/// system, with a minimum of 2.
	pub fn New() -> Self {
		let DefaultWorkerCount = num_cpus::get().max(2).expect("");
		Self { WorkerCount:DefaultWorkerCount, QueueConfiguration:HashMap::new() }
	}

	/// Sets the total number of worker threads for the scheduler's pool.
	///
	/// # Arguments
	/// * `WorkerCount` - The desired number of workers. If `0`, it defaults
	/// to the number of logical CPUs on the system.
	pub fn WithWorkerCount(mut self, WorkerCount:usize) -> Self {
		if WorkerCount == 0 {
			warn!("[SchedulerBuilder] Worker count of 0 is invalid. Defaulting to number of logical CPUs.");
			self.WorkerCount = num_cpus::get().max(2).expect("");
		} else {
			self.WorkerCount = WorkerCount;
		}
		self
	}

	/// Configures a named queue with a specific concurrency limit.
	/// Tasks can later be submitted to this specific queue.
	///
	/// # Arguments
	/// * `QueueName` - The name of the queue (e.g., "DiskIO", "Network").
	/// * `ConcurrencyLimit` - The concurrency configuration for this queue.
	pub fn WithQueue(mut self, QueueName:&str, ConcurrencyLimit:Concurrency) -> Self {
		self.QueueConfiguration.insert(QueueName.to_string(), ConcurrencyLimit);
		self
	}

	/// Builds and starts the `Scheduler` with the specified configuration.
	/// This consumes the builder.
	///
	/// # Returns
	/// A new, running `Scheduler` instance.
	pub fn Build(self) -> Scheduler {
		// The Scheduler's internal `Start` method will receive this
		// configuration and set up the corresponding queues and worker logic.
		Scheduler::Start(self.WorkerCount, self.QueueConfiguration)
	}
}

impl Default for SchedulerBuilder {
	/// Provides a default `SchedulerBuilder` instance.
	fn default() -> Self { Self::New() }
}
