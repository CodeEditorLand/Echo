#[async_trait]
pub trait Trait: Send + Sync {
	async fn Execute(&self, Context: &Life) -> Result<(), ActionError>;

	fn Clone(&self) -> Box<dyn Trait>;
}

#[async_trait]
impl<T: Send + Sync + Clone + 'static> Trait for crate::Struct::Sequence::Action::Struct<T> {
	async fn Execute(&self, Context: &Life) -> Result<(), ActionError> {
		self.Execute(Context).await
	}

	fn Clone(&self) -> Box<dyn Trait> {
		Box::new(self.clone())
	}
}

use async_trait::async_trait;
