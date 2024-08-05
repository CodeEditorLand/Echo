#[derive(Clone, Debug)]
pub struct Struct<T> {
	Value: Arc<Mutex<T>>,
}

impl<T: Serialize> Serialize for Struct<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let guard = self.Value.lock().await;

		T::serialize(&*guard, serializer)
	}
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Struct<T> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let value = T::deserialize(deserializer)?;

		Ok(Struct { Value: Arc::new(Mutex::new(value)) })
	}
}

impl<T: Clone> Struct<T> {
	fn New(InitialValue: T) -> Self {
		Struct { Value: Arc::new(Mutex::new(InitialValue)) }
	}

	async fn Get(&self) -> T {
		self.Value.lock().await.clone()
	}

	async fn Set(&self, NewValue: T) {
		*self.Value.lock().await = NewValue;
	}
}
