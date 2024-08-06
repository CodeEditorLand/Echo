pub struct Struct {
	Formality: Formality::Struct,
}

impl Struct {
	pub fn New() -> Self {
		Self { Formality: Formality::New() }
	}

	pub fn WithSignature(mut self, Signature: ActionSignature) -> Self {
		self.Formality.Sign(Signature);

		self
	}

	pub fn WithFunction<F, Fut>(mut self, Name: &str, Function: F) -> Result<Self, String>
	where
		F: Fn(Vec<serde_json::Value>) -> Fut + Send + Sync + 'static,
		Fut: Future<Output = Result<serde_json::Value, ActionError>> + Send + 'static,
	{
		self.Formality.Add(Name, Function)?;

		Ok(self)
	}

	pub fn Build(self) -> Formality {
		self.Formality
	}
}

pub mod Formality;
use futures::Future;
