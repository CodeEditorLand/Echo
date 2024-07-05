// file_ops_common/src/lib.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub mod Socket;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
	Read { Path: String },
	Write { Path: String, Content: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileOperationResult {
	pub operation: Action,
	pub result: Result<String, String>,
}

#[async_trait]
pub trait Worker: Send + Sync {
	async fn process(&self, task: Action) -> FileOperationResult;
}

pub struct WorkQueue {
	tasks: Arc<Mutex<Vec<Action>>>,
}

impl WorkQueue {
	pub fn new() -> Self {
		WorkQueue { tasks: Arc::new(Mutex::new(Vec::new())) }
	}

	pub async fn Assign(&self, task: Action) {
		self.tasks.lock().await.push(task);
	}

	pub async fn steal(&self) -> Option<Action> {
		self.tasks.lock().await.pop()
	}
}

pub async fn Job(
	worker: Arc<dyn Worker>,
	queue: Arc<WorkQueue>,
	tx: mpsc::Sender<FileOperationResult>,
) {
	loop {
		if let Some(task) = queue.steal().await {
			let result = worker.process(task).await;
			if tx.send(result).await.is_err() {
				break;
			}
		} else {
			tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
		}
	}
}
