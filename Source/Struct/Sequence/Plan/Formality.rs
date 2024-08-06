#[derive(Debug)]
pub struct Struct {
	Signature: DashMap<String, Signature>,
	Function: DashMap<
		String,
		Box<
			dyn Fn(
					Vec<serde_json::Value>,
				) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, Error>> + Send>>
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

	pub fn Add<Function, Future>(
		&mut self,
		Name: &str,
		Function: Function,
	) -> Result<&mut Self, String>
	where
		Function: Fn(Vec<serde_json::Value>) -> Future + Send + Sync + 'static,
		Future: Future<Output = Result<serde_json::Value, Error>> + Send + 'static,
	{
		if !self.Signature.contains_key(Name) {
			return Err(format!("No signature found for function: {}", Name));
		}

		self.Function.insert(
			Name.to_string(),
			Box::new(move |Argument: Vec<serde_json::Value>| {
				Box::pin(Function(Argument))
					as Pin<Box<dyn Future<Output = Result<serde_json::Value, Error>> + Send>>
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
					)
						-> Pin<Box<dyn Future<Output = Result<serde_json::Value, Error>> + Send>>
					+ Send
					+ Sync,
			>,
		>,
	> {
		self.Function.get(Name)
	}
}

use crate::{
	Enum::Sequence::Action::Error::Enum as Error,
	Struct::Sequence::Action::Signature::Struct as Signature,
};
use dashmap::DashMap;
use futures::Future;
