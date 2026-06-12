//! # StealingQueue
//!
//! A generic, priority-aware, work-stealing queue implementation. This module
//! is self-contained and can be used by any scheduler or application to manage
//! and distribute tasks of any type that can be prioritized.

use std::sync::Arc;

use crossbeam_deque::{Injector, Steal, Stealer, Worker};
use rand::RngExt;

/// Contract for types that can be prioritized by the queue.
pub trait Prioritized {
	/// The type of the priority value used by the implementor.
	type Kind: PartialEq + Eq + Copy;

	/// Returns the priority of the item.
	fn Rank(&self) -> Self::Kind;
}

/// Internal priority levels used by the generic queue. Each variant maps to a
/// separate deque managed by the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
	/// Highest-priority deque; executed before all others.
	High,

	/// Default priority level for most standard operations.
	Normal,

	/// Lowest-priority deque; deferred when higher-priority work is available.
	Low,
}

/// Queue components safe to share across all threads.
///
/// Includes global injectors for submitting new tasks from any context and
/// stealers for taking tasks from other workers' deques, organized by priority
/// level.
pub struct Share<TTask> {
	/// Global, multi-producer queues for each priority level.
	pub Injector:(Injector<TTask>, Injector<TTask>, Injector<TTask>),

	/// Shared handles for stealing tasks from each worker's local queue.
	pub Stealer:(Vec<Stealer<TTask>>, Vec<Stealer<TTask>>, Vec<Stealer<TTask>>),
}

/// A generic, priority-aware, work-stealing queue.
///
/// The public-facing entry point for submitting tasks. Generic over any task
/// type `TTask` that implements the `Prioritized` trait.
pub struct StealingQueue<TTask:Prioritized<Kind = Priority>> {
	/// A shared, thread-safe pointer to the queue's shared components.
	Share:Arc<Share<TTask>>,
}

/// All components necessary for a single worker thread to operate.
///
/// Includes the thread-local `Worker` deques, which are not safe to share,
/// making this `Context` the sole owner of a worker's private queues.
pub struct Context<TTask> {
	/// A unique identifier for the worker, used to avoid self-stealing.
	pub Identifier:usize,

	/// Thread-local work queues for each priority level.
	pub Local:(Worker<TTask>, Worker<TTask>, Worker<TTask>),

	/// A reference to the shared components of the entire queue system.
	pub Share:Arc<Share<TTask>>,
}

impl<TTask:Prioritized<Kind = Priority>> StealingQueue<TTask> {
	/// Creates a complete work-stealing queue system.
	///
	/// Initializes all necessary queues — both shared and thread-local — for a
	/// given number of workers.
	///
	/// ## Parameters
	///
	/// * `Count` — Number of workers that will use the queue system.
	///
	/// ## Returns
	///
	/// A tuple containing:
	/// 1. The public-facing `StealingQueue` for submitting new tasks.
	/// 2. A `Vec` of `Context` objects, one for each worker thread to own.
	pub fn Create(Count:usize) -> (Self, Vec<Context<TTask>>) {
		let mut High:Vec<Worker<TTask>> = Vec::with_capacity(Count);

		let mut Normal:Vec<Worker<TTask>> = Vec::with_capacity(Count);

		let mut Low:Vec<Worker<TTask>> = Vec::with_capacity(Count);

		// For each priority level, create a thread-local worker queue and its
		// corresponding shared stealer.
		let StealerHigh:Vec<Stealer<TTask>> = (0..Count)
			.map(|_| {
				let Worker = Worker::new_fifo();

				let Stealer = Worker.stealer();

				High.push(Worker);

				Stealer
			})
			.collect();

		let StealerNormal:Vec<Stealer<TTask>> = (0..Count)
			.map(|_| {
				let Worker = Worker::new_fifo();

				let Stealer = Worker.stealer();

				Normal.push(Worker);

				Stealer
			})
			.collect();

		let StealerLow:Vec<Stealer<TTask>> = (0..Count)
			.map(|_| {
				let Worker = Worker::new_fifo();

				let Stealer = Worker.stealer();

				Low.push(Worker);

				Stealer
			})
			.collect();

		// Bundle all shared components into an Arc for safe sharing.
		let Share = Arc::new(Share {
			Injector:(Injector::new(), Injector::new(), Injector::new()),

			Stealer:(StealerHigh, StealerNormal, StealerLow),
		});

		// Create a unique context for each worker, giving it ownership of its
		// local queues and a reference to the shared components.
		let mut Contexts = Vec::with_capacity(Count);

		for Identifier in 0..Count {
			Contexts.push(Context {
				Identifier,

				Local:(High.remove(0), Normal.remove(0), Low.remove(0)),

				Share:Share.clone(),
			});
		}

		let Queue = Self { Share };

		(Queue, Contexts)
	}

	/// Submits a new task to the appropriate global queue based on its
	/// priority. Thread-safe; can be called from any context.
	pub fn Submit(&self, Task:TTask) {
		match Task.Rank() {
			Priority::High => self.Share.Injector.0.push(Task),

			Priority::Normal => self.Share.Injector.1.push(Task),

			Priority::Low => self.Share.Injector.2.push(Task),
		}
	}
}

impl<TTask> Context<TTask> {
	/// Finds the next available task for the worker to execute.
	///
	/// Implements the complete work-finding logic:
	/// 1. Checks local deques (from high to low priority).
	/// 2. If local deques are empty, attempts to steal from the system (from
	///    high to low priority).
	pub fn Next(&self) -> Option<TTask> {
		self.Local
			.0
			.pop()
			.or_else(|| self.Local.1.pop())
			.or_else(|| self.Local.2.pop())
			.or_else(|| self.Steal(&self.Share.Injector.0, &self.Share.Stealer.0, &self.Local.0))
			.or_else(|| self.Steal(&self.Share.Injector.1, &self.Share.Stealer.1, &self.Local.1))
			.or_else(|| self.Steal(&self.Share.Injector.2, &self.Share.Stealer.2, &self.Local.2))
	}

	/// Attempts to steal a task from a specific priority set.
	///
	/// First tries to steal a batch from the global injector queue for that
	/// priority. If that fails, attempts to steal from a randomly chosen peer
	/// worker to ensure fair distribution and avoid contention hotspots.
	pub fn Steal<'a>(
		&self,

		Injector:&'a Injector<TTask>,

		Stealers:&'a [Stealer<TTask>],

		Local:&'a Worker<TTask>,
	) -> Option<TTask> {
		// First, try to steal a batch from the global injector.
		// `steal_batch_and_pop` is efficient: it moves a batch into our local
		// queue and returns one task immediately if successful.
		if let Steal::Success(Task) = Injector.steal_batch_and_pop(Local) {
			return Some(Task);
		}

		// If the global queue is empty, try stealing from peers.
		let Count = Stealers.len();

		if Count <= 1 {
			// Cannot steal if there are no other workers.
			return None;
		}

		// Allocation-free random iteration: pick a random starting point.
		let mut Rng = rand::rng();

		let Start = Rng.random_range(0..Count);

		// Iterate through all peers starting from the random offset.
		for i in 0..Count {
			let Index = (Start + i) % Count;

			// Don't steal from ourselves.
			if Index == self.Identifier {
				continue;
			}

			if let Steal::Success(Task) = Stealers[Index].steal_batch_and_pop(Local) {
				return Some(Task);
			}
		}

		None
	}
}
