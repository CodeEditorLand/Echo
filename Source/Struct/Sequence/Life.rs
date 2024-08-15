/// Represents the lifecycle context for a sequence of actions.
pub struct Struct {
	/// A thread-safe map of action cycles, identified by string keys.
	/// These cycles represent reusable action functions that can be invoked during execution.
	pub Span: Arc<DashMap<String, crate::Type::Sequence::Action::Cycle>>,

	/// A shared reference to the configuration settings.
	/// This allows for runtime access to various configuration parameters.
	pub Fate: Arc<Config>,

	/// A thread-safe cache for storing arbitrary JSON values.
	/// This cache can be used for temporary storage of data during action execution.
	Cache: Arc<Mutex<DashMap<String, serde_json::Value>>>,

	/// A thread-safe map of production queues, identified by string keys.
	/// Each production queue (represented by `Production`) can hold a series of actions to be executed.
	pub Karma: Arc<DashMap<String, Arc<Production>>>,
}

use dashmap::DashMap;
