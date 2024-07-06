pub mod Yell;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
	Read { Path: String },
	Write { Path: String, Content: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionResult {
	pub Action: Action,
	pub Result: Result<String, String>,
}

#[async_trait::async_trait]
pub trait Worker: Send + Sync {
	async fn Receive(&self, Action: Action) -> ActionResult;
}

pub struct Work {
	Queue: Arc<Mutex<Vec<Action>>>,
}

impl Work {
	pub fn Begin() -> Self {
		Work { Queue: Arc::new(Mutex::new(Vec::new())) }
	}

	pub async fn Assign(&self, Action: Action) {
		self.Queue.lock().await.push(Action);
	}

	pub async fn Execute(&self) -> Option<Action> {
		self.Queue.lock().await.pop()
	}
}

pub async fn Fn(
	Worker: Arc<dyn Worker>,
	Work: Arc<Work>,
	Approval: mpsc::UnboundedSender<ActionResult>,
) {
	loop {
		if let Some(Action) = Work.Execute().await {
			if Approval.send(Worker.Receive(Action).await).await.is_err() {
				break;
			}
		} else {
			// @TODO: Test this for performance
			tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
		}
	}
}
