//! A generic, priority-aware, work-stealing queue implementation.
//!
//! This module is self-contained and can be used by any scheduler or
//! application to manage and distribute tasks of any type that can be
//! prioritized.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use crossbeam_deque::{Injector, Stealer, Worker};
use rand::seq::SliceRandom;

/// Defines a contract for types that can be prioritized by the queue.
pub trait Prioritized {
	/// The type of the priority value used by the implementor.
	type Kind: PartialEq + Eq + Copy;
	/// A method to retrieve the priority of the item.
	fn Rank(&self) -> Self::Kind;
}

/// Defines the internal priority levels used by the generic queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
	High,
	Normal,
	Low,
}

/// Holds the queue components that are safe to share across all threads.
///
/// This includes global injectors for submitting new tasks and stealers for
/// taking tasks from other workers, organized by priority level.
pub struct Share<T> {
	/// Global, multi-producer queues for each priority.
	pub Injector:(Injector<T>, Injector<T>, Injector<T>),
	/// Share handles for stealing tasks from each worker's queue.
	pub Stealer:(Vec<Stealer<T>>, Vec<Stealer<T>>, Vec<Stealer<T>>),
}

/// A generic, priority-aware, work-stealing queue.
///
/// This is the public-facing entry point for submitting tasks. It is generic
/// over any task type `T` that implements the `Prioritized` trait.
pub struct StealingQueue<T:Prioritized<Kind = Priority>> {
	/// A shared, thread-safe pointer to the queue's shared components.
	Share:Arc<Share<T>>,
}

/// Contains all necessary components for a single worker thread to operate.
///
/// This includes the thread-local `Worker` deques, which are not safe to share,
/// making this context object the sole owner of a worker's private queues.
pub struct Context<T> {
	/// A unique identifier for the worker, used to avoid self-stealing.
	pub Identifier:usize,
	/// Thread-local work queues for each priority level.
	pub Local:(Worker<T>, Worker<T>, Worker<T>),
	/// A reference to the shared components of the entire queue system.
	pub Share:Arc<Share<T>>,
}

impl<T:Prioritized<Kind = Priority>> StealingQueue<T> {
	/// Creates a complete work-stealing queue system.
	///
	/// This function initializes all the necessary queues, both shared and
	/// thread-local, for a given number of workers.
	///
	/// Returns a tuple containing:
	/// 1. The public-facing `StealingQueue` for submitting new tasks.
	/// 2. A `Vec` of `Context` objects, one for each worker thread to own.
	pub fn Create(Count:usize) -> (Self, Vec<Context<T>>) {
		let mut High:Vec<Worker<T>> = Vec::with_capacity(Count);
		let mut Normal:Vec<Worker<T>> = Vec::with_capacity(Count);
		let mut Low:Vec<Worker<T>> = Vec::with_capacity(Count);

		// For each priority level, create a thread-local worker queue and its
		// corresponding shared stealer.
		let StealerHigh:Vec<Stealer<T>> = (0..Count)
			.map(|_| {
				let Worker = Worker::new_fifo();
				let Stealer = Worker.stealer();
				High.push(Worker);
				Stealer
			})
			.collect();

		let StealerNormal:Vec<Stealer<T>> = (0..Count)
			.map(|_| {
				let Worker = Worker::new_fifo();
				let Stealer = Worker.stealer();
				Normal.push(Worker);
				Stealer
			})
			.collect();

		let StealerLow:Vec<Stealer<T>> = (0..Count)
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
	/// priority. This method is thread-safe and can be called from any
	/// context.
	pub fn Submit(&self, Task:T) {
		match Task.Rank() {
			Priority::High => self.Share.Injector.0.push(Task),
			Priority::Normal => self.Share.Injector.1.push(Task),
			Priority::Low => self.Share.Injector.2.push(Task),
		}
	}
}

impl<T> Context<T> {
	/// Finds the next available task for the worker to execute.
	// This method implements the complete work-finding logic:
	/// 1. Check local queue (high to low priority).
	/// 2. Steal from the system (high to low priority).
	pub fn Next(&self) -> Option<T> {
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
	/// It first tries to steal a batch from the global injector queue. If that
	/// fails, it attempts to steal from a randomly chosen peer worker to ensure
	/// fair distribution and avoid contention hotspots.
	pub fn Steal<'a>(&self, Injector:&'a Injector<T>, Stealers:&'a [Stealer<T>], Local:&'a Worker<T>) -> Option<T> {
		if Injector.steal_batch_and_pop(Local).is_success() {
			return Local.pop();
		}

		let mut Indices:Vec<usize> = (0..Stealers.len()).collect();
		Indices.shuffle(&mut rand::rng());

		for Index in Indices {
			if Index == self.Identifier {
				continue;
			}
			if Stealers[Index].steal_batch_and_pop(Local).is_success() {
				return Local.pop();
			}
		}
		None
	}
}
