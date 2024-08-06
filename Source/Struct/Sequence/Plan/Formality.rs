#[derive(Debug)]
pub struct Struct {
	Signature: DashMap<String, Signature>,
	Function: DashMap<
		String,
		Box<
			dyn Fn(
					Vec<serde_json::Value>,
				)
					-> Pin<Box<dyn Future<Output = Result<serde_json::Value, ActionError>> + Send>>
				+ Send
				+ Sync,
		>,
	>,
}

impl Struct {
	pub fn New() -> Self {
		Self { Signature: DashMap::new(), Function: DashMap::new() }
	}

	pub fn Sign(&mut self, Signature: Signature) -> &mut Self {
		self.Signature.insert(Signature.Name.clone(), Signature);

		self
	}

	pub fn Add<F, Fut>(&mut self, Name: &str, Function: F) -> Result<&mut Self, String>
	where
		F: Fn(Vec<serde_json::Value>) -> Fut + Send + Sync + 'static,
		Fut: Future<Output = Result<serde_json::Value, ActionError>> + Send + 'static,
	{
		if !self.Signature.contains_key(Name) {
			return Err(format!("No signature found for function: {}", Name));
		}

		self.Function.insert(
			Name.to_string(),
			Box::new(move |Args: Vec<serde_json::Value>| {
				Box::pin(Function(Args))
					as Pin<Box<dyn Future<Output = Result<serde_json::Value, ActionError>> + Send>>
			}),
		);

		Ok(self)
	}

	pub fn Remove(
		&self,
		Name: &str,
	) -> Option<
		impl Borrow<
			Box<
				dyn Fn(
						Vec<serde_json::Value>,
					) -> Pin<
						Box<dyn Future<Output = Result<serde_json::Value, ActionError>> + Send>,
					> + Send
					+ Sync,
			>,
		>,
	> {
		self.Function.get(Name)
	}
}

use crate::Struct::Sequence::Action::Signature::Struct as Signature;
use dashmap::DashMap;
use futures::Future;

