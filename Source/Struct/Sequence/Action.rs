#[derive(Clone, Debug)]
pub struct Action<T: Send + Sync> {
	pub Metadata: VectorDatabase,
	pub Content: T,
	pub LicenseSignal: Signal<bool>,
	pub Plan: Arc<Formality>,
}

impl<T: Send + Sync + Serialize> Serialize for Action<T> {
	fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		unimplemented!()
	}
}

impl<'de, T: Send + Sync + Deserialize<'de>> Deserialize<'de> for Action<T> {
	fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		unimplemented!()
	}
}

impl<T: Send + Sync> Action<T> {
	pub fn New(ActionType: &str, Content: T, Plan: Arc<Formality>) -> Self {
		let mut Metadata = VectorDatabase::New();

		Metadata.Insert("ActionType".to_string(), serde_json::json!(ActionType));

		Metadata.Insert("License".to_string(), serde_json::json!("valid"));

		Action { Metadata, Content, LicenseSignal: Signal::New(true), Plan }
	}

	pub fn WithMetadata(mut self, Key: &str, Value: serde_json::Value) -> Self {
		self.Metadata.Insert(Key.to_string(), Value);

		self
	}

	pub async fn Execute(&self, Context: &Life) -> Result<(), ActionError> {
		let ActionType = self
			.Metadata
			.Get("ActionType")
			.await
			.ok_or_else(|| ActionError::ExecutionError("ActionType not found".to_string()))?
			.as_str()
			.ok_or_else(|| ActionError::ExecutionError("ActionType is not a string".to_string()))?
			.to_string();

		info!("Executing action: {}", ActionType);

		self.CheckLicense().await?;

		self.HandleDelay().await?;

		self.ExecuteHooks(Context).await?;

		self.ExecuteFunction(&ActionType).await?;

		self.HandleNextAction(Context).await?;

		Ok(())
	}

	async fn CheckLicense(&self) -> Result<(), ActionError> {
		if !self.LicenseSignal.Get().await {
			return Err(ActionError::InvalidLicense("Invalid action license".to_string()));
		}

		Ok(())
	}

	async fn HandleDelay(&self) -> Result<(), ActionError> {
		if let Some(Delay) = self.Metadata.Get("Delay").await {
			let Delay = Duration::from_secs(Delay.as_u64().unwrap_or(0));

			sleep(Delay).await;
		}

		Ok(())
	}

	async fn ExecuteHooks(&self, Context: &Life) -> Result<(), ActionError> {
		if let Some(Hooks) = self.Metadata.Get("Hooks").await {
			for Hook in Hooks.as_array().unwrap_or(&Vec::new()) {
				if let Some(HookFn) = Context.Span.get(Hook.as_str().unwrap_or("")) {
					HookFn()?;
				}
			}
		}

		Ok(())
	}

	async fn ExecuteFunction(&self, ActionType: &str) -> Result<(), ActionError> {
		if let Some(Function) = self.Plan.Remove(ActionType) {
			self.Result(Function.borrow()(self.Argument().await?).await?).await?;
		} else {
			return Err(ActionError::ExecutionError(format!(
				"No function found for action type: {}",
				ActionType
			)));
		}

		Ok(())
	}

	async fn HandleNextAction(&self, Context: &Life) -> Result<(), ActionError> {
		if let Some(NextAction) = self.Metadata.Get("NextAction").await {
			let NextAction: Action<T> =
				serde_json::from_value(NextAction.clone()).map_err(|e| {
					ActionError::ExecutionError(format!("Failed to parse NextAction: {}", e))
				})?;

			NextAction.Execute(Context).await?;
		}

		Ok(())
	}

	async fn Argument(&self) -> Result<Vec<serde_json::Value>, ActionError> {
		Ok(vec![])
	}

	async fn Result(&self, Result: serde_json::Value) -> Result<(), ActionError> {
		Ok(())
	}
}

pub mod Error;
pub mod Signature;
use log::info;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt::Debug, sync::Arc, time::Duration};
