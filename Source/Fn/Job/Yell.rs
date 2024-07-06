use crate::Fn::Job::{Action, ActionResult, Work};

use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message;

pub async fn Fn(
	Order: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
	Work: Arc<Work>,
	Receipt: Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<ActionResult>>>,
) {
	let (mut Write, mut Read) = Order.split();

	loop {
		tokio::select! {
			Some(Shout) = Read.next() => {
				if let Ok(Message::Text(text)) = Shout {
					if let Ok(Action) = serde_json::from_str::<Action>(&text) {
						Work.Assign(Action).await;
					}
				}
			}

			Some(Shout) = Receipt. => {
				if Write.send(Message::Text(serde_json::to_string(&Shout).unwrap())).await.is_err() {
					break;
				}
			}

			else => break,
		}
	}
}
