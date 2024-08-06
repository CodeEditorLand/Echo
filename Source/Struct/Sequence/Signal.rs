#[derive(Clone, Debug)]
pub struct Struct<T>(Arc<Mutex<T>>);

impl<T> Struct<T> {
	pub fn New(Value: T) -> Self {
		Struct(Arc::new(Mutex::new(Value)))
	}

	pub async fn Get(&self) -> T
	where
		T: Clone,
	{
		self.0.lock().await.clone()
	}

	pub async fn Set(&self, To: T) {
		*self.0.lock().await = To;
	}
}
