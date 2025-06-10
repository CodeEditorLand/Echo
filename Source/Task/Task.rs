use std::{future::Future, pin::Pin};

use super::Priority::Enum;
use crate::Queue::StealingQueue::{Prioritized, Priority};

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct Struct {
	pub Future:BoxedFuture,

	pub Priority:Enum,
}

impl Struct {
	pub fn New<F>(FutureInstance:F, PriorityValue:Enum) -> Self
	where
		F: Future<Output = ()> + Send + 'static, {
		Self { Future:Box::pin(FutureInstance), Priority:PriorityValue }
	}
}

impl Prioritized for Struct {
	type P = Priority;

	fn GetPriority(&self) -> Self::P {
		match self.Priority {
			Enum::High => Priority::High,

			Enum::Normal => Priority::Normal,

			Enum::Low => Priority::Low,
		}
	}
}
