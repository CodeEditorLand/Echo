// file_ops_common/src/lib.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub mod websocket;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileOperation {
	Read { path: String },
	Write { path: String, content: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileOperationResult {
	pub operation: FileOperation,
	pub result: Result<String, String>,
}

#[async_trait]
pub trait Worker: Send + Sync {
	async fn process(&self, task: FileOperation) -> FileOperationResult;
}

pub struct WorkQueue {
	tasks: Arc<Mutex<Vec<FileOperation>>>,
}

impl WorkQueue {
	pub fn new() -> Self {
		WorkQueue { tasks: Arc::new(Mutex::new(Vec::new())) }
	}

	pub async fn push(&self, task: FileOperation) {
		let mut tasks = self.tasks.lock().await;
		tasks.push(task);
	}

	pub async fn steal(&self) -> Option<FileOperation> {
		let mut tasks = self.tasks.lock().await;
		tasks.pop()
	}
}

pub async fn worker_loop(
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

// file_ops_common/src/websocket.rs

use super::{FileOperation, FileOperationResult, WorkQueue};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub async fn Fn(
	stream: WebSocketStream<TcpStream>,
	queue: Arc<WorkQueue>,
	mut rx: mpsc::Receiver<FileOperationResult>,
) {
	let (mut write, mut read) = stream.split();

	loop {
		tokio::select! {
			Some(message) = read.next() => {
				if let Ok(Message::Text(text)) = message {
					if let Ok(operation) = serde_json::from_str::<FileOperation>(&text) {
						queue.push(operation).await;
					}
				}
			}
			Some(result) = rx.recv() => {
				let message = serde_json::to_string(&result).unwrap();
				if write.send(Message::Text(message)).await.is_err() {
					break;
				}
			}
			else => break,
		}
	}
}
