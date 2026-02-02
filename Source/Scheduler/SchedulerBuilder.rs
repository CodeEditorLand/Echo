//! # SchedulerBuilder
//!
//! Defines the fluent builder for creating and configuring a `Scheduler`.

use std::collections::HashMap;

use log::warn;

use crate::Scheduler::Scheduler::Scheduler;

/// Defines concurrency limits for named queues (for future use).
#[derive(Debug, Clone, Copy)]
pub enum Concurrency {
	/// Specifies a maximum number of concurrent tasks for a queue.
	Limit(usize),

	/// Allows an unlimited number of concurrent tasks for a queue.
	Unlimited,
}

/// A fluent builder for creating a `Scheduler` instance.
///
/// This pattern provides a clear and readable API for configuring the scheduler
/// before it is constructed. It is the primary entry point for using the `Echo`
/// library.
pub struct SchedulerBuilder {
	/// The number of worker threads to be spawned in the scheduler's pool.
	Count:usize,

	/// Configuration for named queues with concurrency limits (for future
	/// use).
	Configuration:HashMap<String, Concurrency>,
}

impl SchedulerBuilder {
	/// Creates a new `SchedulerBuilder` with default settings.
	///
	/// By default, the worker count is set to the number of logical CPUs on the
	/// system, with a minimum of two workers to ensure work-stealing is
	/// viable.
	pub fn Create() -> Self {
		let Default = num_cpus::get().max(2);

		Self { Count:Default, Configuration:HashMap::new() }
	}

	/// Sets the total number of worker threads for the scheduler pool.
	///
	/// If `Count` is `0`, it defaults to the number of logical CPUs.
	pub fn WithWorkerCount(mut self, Count:usize) -> Self {
		if Count == 0 {
			warn!("[SchedulerBuilder] Worker count of 0 is invalid. Defaulting to logical CPUs.");

			self.Count = num_cpus::get().max(2);
		} else {
			self.Count = Count;
		}

		self
	}

	/// Configures a named queue with a specific concurrency limit (for future
	/// use).
	pub fn WithQueue(mut self, Name:&str, Limit:Concurrency) -> Self {
		self.Configuration.insert(Name.to_string(), Limit);

		self
	}

	/// Builds and starts the `Scheduler` with the specified configuration.
	///
	/// This method consumes the builder and returns a new, running `Scheduler`.
	pub fn Build(self) -> Scheduler { Scheduler::Create(self.Count, self.Configuration) }
}

impl Default for SchedulerBuilder {
	/// Provides a default `SchedulerBuilder` instance.
	fn default() -> Self { Self::Create() }
}
