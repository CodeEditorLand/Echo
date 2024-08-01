/// Asynchronously processes actions from a work queue and sends the results to an approval channel.
///
/// # Arguments
///
/// * `Site` - An `Arc` reference to a type that implements the `Worker` trait. This is used to process the actions.
/// * `Work` - An `Arc` reference to a `Work` instance that contains the queue of actions to be processed.
/// * `Approval` - An unbounded sender channel to send the results of the processed actions.
///
/// # Behavior
///
/// This function runs an infinite loop where it continuously checks for actions in the `Work` queue.
/// If an action is found, it is processed by the `Site` and the result is sent to the `Approval` channel.
/// If sending the result fails, the loop breaks. If no action is found, the function sleeps for 100 milliseconds
/// before checking again.
pub async fn Fn(
	Site: Arc<dyn crate::Trait::Job::Worker::Trait>,
	Work: Arc<crate::Struct::Job::Work::Struct>,
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

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod Yell;
