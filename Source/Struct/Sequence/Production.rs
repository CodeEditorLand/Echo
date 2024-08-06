pub struct Struct {
	Line: Arc<Mutex<VecDeque<Box<dyn ActionTrait>>>>,
}

impl Struct {
	pub fn New() -> Self {
		Struct { Line: Arc::new(Mutex::new(VecDeque::new())) }
	}

	pub async fn Do(&self) -> Option<Box<dyn ActionTrait>> {
		self.Line.lock().await.pop_front()
	}

	pub async fn Take(&self, Action: Box<dyn ActionTrait>) {
		self.Line.lock().await.push_back(Action);
	}
}

use std::{collections::VecDeque, sync::Arc};
