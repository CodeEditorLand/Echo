// @module StealingQueue
// @description A generic, priority-aware, work-stealing queue implementation.
// This module is self-contained and can be used by any scheduler or application
// to manage and distribute tasks of any type `T`.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use crossbeam_deque::{Injector, Stealer, Worker as WorkerDeque};
use rand::seq::SliceRandom;

// The task must have a way to specify its priority. We define a trait for this.
pub trait Prioritized {
	type P: PartialEq + Eq + Copy;

	fn GetPriority(&self) -> Self::P;
}

// A simple enum for the library to use. The consumer's task must map to this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
	High,

	Normal,

	Low,
}

// The parts of the queue that can be safely shared across all threads.
struct Shared<T> {
	// High, Normal, Low
	Injector:(Injector<T>, Injector<T>, Injector<T>),

	// High, Normal, Low
	Stealer:(Vec<Stealer<T>>, Vec<Stealer<T>>, Vec<Stealer<T>>),
}

/// The public-facing work-stealing queue. It is generic over the task type `T`.
/// This object is held by the task submitter.
pub struct StealingQueue<T:Prioritized<P = Priority>> {
	Shared:Arc<Shared<T>>,
}

/// A context object that contains everything a single worker thread needs to
/// operate. This includes its own private, thread-local deques.
pub struct WorkerContext<T> {
	Id:usize,

	// High, Normal, Low
	Local:(WorkerDeque<T>, WorkerDeque<T>, WorkerDeque<T>),

	Shared:Arc<Shared<T>>,
}

impl<T:Prioritized<P = Priority>> StealingQueue<T> {
	/// Creates a new work-stealing queue system.
	///
	/// Returns a tuple containing:
	/// 1. The `StealingQueue` for submitting tasks.
	/// 2. A `Vec` of `WorkerContext`s, one for each worker to be spawned.
	pub fn New(Count:usize) -> (Self, Vec<WorkerContext<T>>) {
		let mut High:Vec<WorkerDeque<T>> = Vec::with_capacity(Count);

		let mut Normal:Vec<WorkerDeque<T>> = Vec::with_capacity(Count);

		let mut Low:Vec<WorkerDeque<T>> = Vec::with_capacity(Count);

		// --- FIX: Use the documented API for creating Worker/Stealer pairs ---
		let StealerHigh:Vec<Stealer<T>> = (0..Count)
			.map(|_| {
				// 1. Create the Worker.
				let Worker = WorkerDeque::new_fifo();

				// 2. Get its Stealer.
				let Stealer = Worker.stealer();

				// 3. Store the Worker part.
				High.push(Worker);

				// 4. Return the Stealer part.
				Stealer
			})
			.collect();

		let StealerNormal:Vec<Stealer<T>> = (0..Count)
			.map(|_| {
				let Worker = WorkerDeque::new_fifo();

				let Stealer = Worker.stealer();

				Normal.push(Worker);

				Stealer
			})
			.collect();

		let StealerLow:Vec<Stealer<T>> = (0..Count)
			.map(|_| {
				let Worker = WorkerDeque::new_fifo();

				let Stealer = Worker.stealer();

				Low.push(Worker);

				Stealer
			})
			.collect();

		let Shared = Arc::new(Shared {
			Injector:(Injector::new(), Injector::new(), Injector::new()),

			Stealer:(StealerHigh, StealerNormal, StealerLow),
		});

		let mut Context = Vec::with_capacity(Count);

		for Id in 0..Count {
			// We use remove(0) because we built the Vecs in order and need to consume them.
			Context.push(WorkerContext {
				Id,

				Local:(High.remove(0), Normal.remove(0), Low.remove(0)),

				Shared:Shared.clone(),
			});
		}

		let Queue = Self { Shared };

		(Queue, Context)
	}

	/// Submits a new task to the queue.
	/// This is thread-safe and can be called from anywhere.
	pub fn Submit(&self, Task:T) {
		match Task.GetPriority() {
			Priority::High => self.Shared.Injector.0.push(Task),

			Priority::Normal => self.Shared.Injector.1.push(Task),

			Priority::Low => self.Shared.Injector.2.push(Task),
		}
	}
}

impl<T> WorkerContext<T> {
	/// Finds the next available task for this worker.
	/// Implements the full priority-aware, work-stealing logic.
	pub fn NextTask(&self) -> Option<T> {
		// Pop from local High
		self.Local.0.pop()
			 // Pop from local Normal
			.or_else(|| self.Local.1.pop())
			 // Pop from local Low
			.or_else(|| self.Local.2.pop())
			 // Steal High
			.or_else(|| self.Steal(&self.Shared.Injector.0, &self.Shared.Stealer.0, &self.Local.0))
			 // Steal Normal
			.or_else(|| self.Steal(&self.Shared.Injector.1, &self.Shared.Stealer.1, &self.Local.1))
			 // Steal Low
			.or_else(|| self.Steal(&self.Shared.Injector.2, &self.Shared.Stealer.2, &self.Local.2))
	}

	fn Steal<'a>(&self, Injector:&'a Injector<T>, Stealer:&'a [Stealer<T>], Local:&'a WorkerDeque<T>) -> Option<T> {
		if Injector.steal_batch_and_pop(Local).is_success() {
			return Local.pop();
		}

		let mut Index:Vec<usize> = (0..Stealer.len()).collect();

		Index.shuffle(&mut rand::rng());

		for i in Index {
			if i == self.Id {
				continue;
			}

			if Stealer[i].steal_batch_and_pop(Local).is_success() {
				return Local.pop();
			}
		}

		None
	}
}
