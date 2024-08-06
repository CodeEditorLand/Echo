#[derive(Clone, Debug)]
pub struct Struct {
	Entry: DashMap<String, serde_json::Value>,
}

impl Struct {
	pub fn New() -> Self {
		Self { Entry: DashMap::new() }
	}

	pub fn Insert(&mut self, Key: String, Value: serde_json::Value) {
		self.Entry.insert(Key, Value);
	}

	pub async fn Get(&self, Key: &str) -> Option<serde_json::Value> {
		self.Entry.get(Key).map(|v| v.value().clone())
	}
}

use dashmap::DashMap;
