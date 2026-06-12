//! # SchedulerBuilder
//!
//! Defines the fluent builder for creating and configuring a `Scheduler`.

use std::collections::HashMap;

use log::warn;

use crate::Scheduler::Scheduler::Scheduler;

/// Defines concurrency limits for named queues (for future use).
#[derive(Debug, Clone, Copy)]
pub enum Concurrency {
	/// Limit — Restricts execution to a maximum number of concurrent tasks for
	/// a named queue.
	Limit(usize),

	/// Unlimited — Permits an arbitrary number of concurrent tasks for a named
	/// queue.
	Unlimited,
}

/// A fluent builder for creating a `Scheduler` instance.
///
/// Provides a clear, readable API for configuring the scheduler before it is
/// constructed. This is the primary entry point for using the Echo library.
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
	/// The worker count defaults to the number of logical CPUs on the system,
	/// with a minimum of two workers to ensure work-stealing is viable.
	///
	/// ## Returns
	///
	/// A new `SchedulerBuilder` instance configured with system-appropriate
	/// defaults.
	pub fn Create() -> Self {
		let Default = num_cpus::get().max(2);

		Self { Count:Default, Configuration:HashMap::new() }
	}

	/// Sets the total number of worker threads for the scheduler pool.
	///
	/// If `Count` is `0`, it defaults to the number of logical CPUs.
	///
	/// ## Parameters
	///
	/// * `Count` — Number of worker threads to spawn.
	///
	/// ## Returns
	///
	/// The builder with the updated worker count, enabling method chaining.
	pub fn WithWorkerCount(mut self, Count:usize) -> Self {
		if Count == 0 {
			warn!("[SchedulerBuilder] Worker count of 0 is invalid. Defaulting to logical CPUs.");

			self.Count = num_cpus::get().max(2);
		} else {
			self.Count = Count;
		}

		self
	}

	/// Configures a named queue with a specific concurrency limit (reserved for
	/// future use).
	///
	/// ## Parameters
	///
	/// * `Name` — Queue name.
	/// * `Limit` — Concurrency limit for the queue.
	///
	/// ## Returns
	///
	/// The builder with the queue configuration appended, enabling method
	/// chaining.
	pub fn WithQueue(mut self, Name:&str, Limit:Concurrency) -> Self {
		self.Configuration.insert(Name.to_string(), Limit);

		self
	}

	/// Builds and starts the `Scheduler` with the specified configuration.
	///
	/// Consumes the builder and returns a new, running `Scheduler`.
	///
	/// ## Returns
	///
	/// A running `Scheduler` instance.
	pub fn Build(self) -> Scheduler {
		// Telemetry: emit one `land:echo:scheduler:start` per built
		// scheduler so the Boot dashboard can track worker-pool size
		// across processes that link Echo (today: Mountain). No-op in
		// release / when `Capture=false`.
		let WorkerCount = format!("{}", self.Count);

		CommonLibrary::Telemetry::CaptureEvent::Fn(
			"land:echo:scheduler:start",
			Some(vec![("worker_count", WorkerCount.as_str())]),
		);

		Scheduler::Create(self.Count, self.Configuration)
	}
}

impl Default for SchedulerBuilder {
	/// Provides a default `SchedulerBuilder` instance via
	/// `SchedulerBuilder::Create`.
	fn default() -> Self { Self::Create() }
}
