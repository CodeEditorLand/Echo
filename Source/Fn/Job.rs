#![allow(non_snake_case)]

pub mod Yell;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

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
	Site: Arc<dyn Worker>,
	Work: Arc<Work>,
	Approval: tokio::sync::mpsc::UnboundedSender<ActionResult>,
) {
	loop {
		if let Some(Action) = Work.Execute().await {
			if Approval.send(Site.Receive(Action).await).is_err() {
				break;
			}
		} else {
			tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
		}
	}
}
