/// Represents a work queue that holds actions to be processed.
pub struct Struct {
	Queue: Arc<Mutex<Vec<Action>>>,
}

impl Struct {
	/// Creates a new `Work` instance with an empty queue.
	///
	/// # Returns
	///
	/// A new `Work` instance
	pub fn Fn() -> Self {
		Struct { Queue: Arc::new(Mutex::new(Vec::new())) }
	}

	/// Assigns a new action to the work queue.
	///
	/// # Arguments
	///
	/// * `Action` - The action to be added to the queue.
	pub async fn Assign(&self, Action: Action) {
		self.Queue.lock().await.push(Action);
	}

	/// Executes the next action from the work queue.
	///
	/// # Returns
	///
	/// An `Option` containing the next action if available, or `None` if the queue is empty.
	pub async fn Execute(&self) -> Option<Action> {
		self.Queue.lock().await.pop()
	}
}
