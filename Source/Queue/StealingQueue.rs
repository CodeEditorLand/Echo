#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use crossbeam_deque::{Injector, Stealer, Worker};
use rand::seq::SliceRandom;

pub trait Prioritized {
	type P: PartialEq + Eq + Copy;

	fn GetPriority(&self) -> Self::P;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
	High,

	Normal,

	Low,
}

struct Share<T> {
	Injector:(Injector<T>, Injector<T>, Injector<T>),

	Stealer:(Vec<Stealer<T>>, Vec<Stealer<T>>, Vec<Stealer<T>>),
}

pub struct StealingQueue<T:Prioritized<P = Priority>> {
	Share:Arc<Share<T>>,
}

pub struct Context<T> {
	pub Identifier:usize,

	Local:(Worker<T>, Worker<T>, Worker<T>),

	Share:Arc<Share<T>>,
}

impl<T:Prioritized<P = Priority>> StealingQueue<T> {
	pub fn New(Count:usize) -> (Self, Vec<Context<T>>) {
		let mut High:Vec<Worker<T>> = Vec::with_capacity(Count);

		let mut Normal:Vec<Worker<T>> = Vec::with_capacity(Count);

		let mut Low:Vec<Worker<T>> = Vec::with_capacity(Count);

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

		let Shared = Arc::new(Share {
			Injector:(Injector::new(), Injector::new(), Injector::new()),

			Stealer:(StealerHigh, StealerNormal, StealerLow),
		});

		let mut Context = Vec::with_capacity(Count);

		for Id in 0..Count {
			Context.push(Context {
				Identifier:Id,

				Local:(High.remove(0), Normal.remove(0), Low.remove(0)),

				Share:Shared.clone(),
			});
		}

		let Queue = Self { Share:Shared };

		(Queue, Context)
	}

	pub fn Submit(&self, Task:T) {
		match Task.GetPriority() {
			Priority::High => self.Share.Injector.0.push(Task),

			Priority::Normal => self.Share.Injector.1.push(Task),

			Priority::Low => self.Share.Injector.2.push(Task),
		}
	}
}

impl<T> Context<T> {
	pub fn NextTask(&self) -> Option<T> {
		self.Local
			.0
			.pop()
			.or_else(|| self.Local.1.pop())
			.or_else(|| self.Local.2.pop())
			.or_else(|| self.Steal(&self.Share.Injector.0, &self.Share.Stealer.0, &self.Local.0))
			.or_else(|| self.Steal(&self.Share.Injector.1, &self.Share.Stealer.1, &self.Local.1))
			.or_else(|| self.Steal(&self.Share.Injector.2, &self.Share.Stealer.2, &self.Local.2))
	}

	fn Steal<'a>(&self, Injector:&'a Injector<T>, Stealer:&'a [Stealer<T>], Local:&'a Worker<T>) -> Option<T> {
		if Injector.steal_batch_and_pop(Local).is_success() {
			return Local.pop();
		}

		let mut Index:Vec<usize> = (0..Stealer.len()).collect();

		Index.shuffle(&mut rand::rng());

		for i in Index {
			if i == self.Identifier {
				continue;
			}

			if Stealer[i].steal_batch_and_pop(Local).is_success() {
				return Local.pop();
			}
		}

		None
	}
}
