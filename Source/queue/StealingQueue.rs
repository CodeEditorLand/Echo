//! Defines a high-performance, priority-aware, work-stealing deque for
//! distributing tasks among scheduler workers.

use crossbeam_deque::{Injector, Stealer, Worker};
use rand::seq::SliceRandom;

use crate::task::{Priority, Task};

/// A container for a set of queues for a single priority level.
struct PriorityQueueSet {
	GlobalInjector:Injector<Task>,
	WorkerQueue:Vec<Worker<Task>>,
	Stealer:Vec<Stealer<Task>>,
}

impl PriorityQueueSet {
	/// Creates a new set of queues for a given number of workers.
	fn New(NumberOfWorker:usize) -> Self {
		let WorkerQueue:Vec<Worker<Task>> = (0..NumberOfWorker).map(|_| Worker::new_fifo()).collect();
		Self {
			GlobalInjector:Injector::new(),
			Stealer:WorkerQueue.iter().map(|w| w.stealer()).collect(),
			WorkerQueue,
		}
	}
}

/// A collection of worker deques that supports priority-aware work-stealing.
///
/// This struct holds three distinct sets of queues, one for each priority level
/// (`High`, `Normal`, `Low`). When a worker needs a task, it always checks for
/// higher-priority work before considering lower-priority work.
pub(crate) struct StealingQueue {
	High:PriorityQueueSet,
	Normal:PriorityQueueSet,
	Low:PriorityQueueSet,
}

impl StealingQueue {
	/// Creates a new `StealingQueue` with a dedicated set of queues for each
	/// priority level.
	pub fn New(NumberOfWorker:usize) -> Self {
		Self {
			High:PriorityQueueSet::New(NumberOfWorker),
			Normal:PriorityQueueSet::New(NumberOfWorker),
			Low:PriorityQueueSet::New(NumberOfWorker),
		}
	}

	/// Submits a task to the appropriate global injection queue based on its
	/// priority.
	pub fn Push(&self, Task:Task) {
		match Task.Priority {
			Priority::High => self.High.GlobalInjector.push(Task),
			Priority::Normal => self.Normal.GlobalInjector.push(Task),
			Priority::Low => self.Low.GlobalInjector.push(Task),
		}
	}

	/// Attempts to find a task for a given worker, always prioritizing
	/// `High` > `Normal` > `Low`.
	pub fn StealForWorker(&self, WorkerId:usize) -> Option<Task> {
		self.FindTaskInSet(&self.High, WorkerId)
			.or_else(|| self.FindTaskInSet(&self.Normal, WorkerId))
			.or_else(|| self.FindTaskInSet(&self.Low, WorkerId))
	}

	/// Implements the core work-finding logic for a specific priority level.
	/// It first checks the worker's local queue, then attempts to steal.
	fn FindTaskInSet(&self, Set:&PriorityQueueSet, WorkerId:usize) -> Option<Task> {
		Set.WorkerQueue[WorkerId]
			.pop()
			.or_else(|| self.StealFromSetGlobalOrPeer(Set, WorkerId))
	}

	/// Attempts to steal work for a specific priority level, first from the
	/// global queue, then from peer workers.
	fn StealFromSetGlobalOrPeer(&self, Set:&PriorityQueueSet, WorkerId:usize) -> Option<Task> {
		// Try stealing from the global injector for this priority set.
		if Set.GlobalInjector.steal_batch_and_pop(&Set.WorkerQueue[WorkerId]).is_success() {
			return Set.WorkerQueue[WorkerId].pop();
		}

		// Try stealing from peers for this priority set. We shuffle the indices
		// to ensure fairness and avoid contention hotspots.
		let mut ShuffledIndex:Vec<usize> = (0..Set.Stealer.len()).collect();
		ShuffledIndex.shuffle(&mut rand::thread_rng());

		for Index in ShuffledIndex {
			if Index == WorkerId {
				continue; // Don't steal from ourselves.
			}
			if Set.Stealer[Index].steal_batch_and_pop(&Set.WorkerQueue[WorkerId]).is_success() {
				return Set.WorkerQueue[WorkerId].pop();
			}
		}

		None
	}
}
