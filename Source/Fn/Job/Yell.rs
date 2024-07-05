use super::{Action, ActionResult, WorkQueue};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub async fn Fn(
	stream: WebSocketStream<TcpStream>,
	queue: Arc<WorkQueue>,
	mut rx: mpsc::Receiver<ActionResult>,
) {
	let (mut write, mut read) = stream.split();

	loop {
		tokio::select! {
			Some(message) = read.next() => {
				if let Ok(Message::Text(text)) = message {
					if let Ok(operation) = serde_json::from_str::<Action>(&text) {
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
