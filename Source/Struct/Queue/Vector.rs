#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Struct {
	Entry: DashMap<String, Signal<serde_json::Value>>,
}

impl Struct {
	pub fn New() -> Self {
		Struct { Entry: DashMap::new() }
	}

	pub fn Insert(&mut self, Key: String, Value: serde_json::Value) {
		self.Entry.insert(Key, Signal::New(Value));
	}

	pub async fn Get(&self, Key: &str) -> Option<serde_json::Value> {
		self.Entry.get(Key).map(|signal| signal.Get().await)
	}
}
