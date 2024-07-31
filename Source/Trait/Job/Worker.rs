/// A trait that defines the behavior for processing actions.
///
/// Types that implement this trait must be able to handle actions asynchronously.
#[async_trait::async_trait]
pub trait Trait: Send + Sync {
	/// Processes a given action and returns the result.
	///
	/// # Arguments
	///
	/// * `Action` - The action to be processed.
	///
	/// # Returns
	///
	/// An `ActionResult` containing the result of the action.
	async fn Receive(&self, Action: Action) -> ActionResult;
}
