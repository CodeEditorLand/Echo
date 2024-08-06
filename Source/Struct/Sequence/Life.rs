pub struct Struct {
	pub Span: Arc<DashMap<String, StartEnd>>,
	pub Fate: Arc<Config>,
	Cache: Arc<Mutex<DashMap<String, serde_json::Value>>>,
	pub Karma: Arc<DashMap<String, Arc<Production>>>,
}

use dashmap::DashMap;
