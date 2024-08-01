/// Asynchronously processes WebSocket messages and actions from a work queue.
///
/// # Arguments
///
/// * `Order` - A WebSocket stream used for reading and writing messages.
/// * `Work` - An `Arc` reference to a `Work` instance that contains the queue of actions to be processed.
/// * `Receipt` - An `Arc` reference to a mutex-protected unbounded receiver channel for receiving action results.
///
/// # Behavior
///
/// This function runs an infinite loop where it uses `tokio::select!` to concurrently:
/// 1. Read messages from the WebSocket stream. If a message is received and successfully parsed into an `Action`,
///    it is assigned to the work queue.
/// 2. Receive action results from the `Receipt` channel and send them back through the WebSocket stream.
///
/// If sending a message through the WebSocket stream fails, the loop breaks.
pub async fn Fn(
	Order: crate::Type::Job::Yell::Order::Type,
	Work: Arc<crate::Struct::Job::Work::Struct>,
	Receipt: Arc<crate::Type::Job::Yell::Receipt::Type>,
) {
	let (mut Write, mut Read) = Order.split();

	loop {
		tokio::select! {
			Some(Shout) = Read.next() => {
				if let Ok(Message::Text(Shout)) = Shout {
					if let Ok(Action) = serde_json::from_str::<crate::Struct::Job::Action::Struct>(&Shout) {
						Work.Assign(Action).await;
					}
				}
			}

			Some(Shout) = async {
				Receipt.lock().await.recv().await
			} => {
				if Write.send(Message::Text(serde_json::to_string(&Shout).unwrap())).await.is_err() {
					break;
				}
			}

			else => break,
		}
	}
}

use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message;
