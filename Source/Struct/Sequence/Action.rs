#[derive(Clone, Debug)]
pub struct Struct<T: Send + Sync> {
	pub Metadata: Vector,
	pub Content: T,
	pub LicenseSignal: Signal<bool>,
	pub Plan: Arc<Formality>,
}

impl<T: Send + Sync + Serialize> Serialize for Struct<T> {
	fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		unimplemented!()
	}
}

impl<'de, T: Send + Sync + Deserialize<'de>> Deserialize<'de> for Struct<T> {
	fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		unimplemented!()
	}
}

impl<T: Send + Sync> Struct<T> {
	pub fn New(ActionType: &str, Content: T, Plan: Arc<Formality>) -> Self {
		let mut Metadata = Vector::New();

		Metadata.Insert("ActionType".to_string(), serde_json::json!(ActionType));

		Metadata.Insert("License".to_string(), serde_json::json!("valid"));

		Struct { Metadata, Content, LicenseSignal: Signal::New(true), Plan }
	}

	pub fn WithMetadata(mut self, Key: &str, Value: serde_json::Value) -> Self {
		self.Metadata.Insert(Key.to_string(), Value);

		self
	}

	pub async fn Execute(&self, Context: &Life) -> Result<(), Error> {
		let ActionType = self
			.Metadata
			.Get("ActionType")
			.await
			.ok_or_else(|| Error::ExecutionError("ActionType not found".to_string()))?
			.as_str()
			.ok_or_else(|| Error::ExecutionError("ActionType is not a string".to_string()))?
			.to_string();

		info!("Executing action: {}", ActionType);

		self.CheckLicense().await?;

		self.HandleDelay().await?;

		self.ExecuteHooks(Context).await?;

		self.ExecuteFunction(&ActionType).await?;

		self.HandleNextAction(Context).await?;

		Ok(())
	}

	async fn CheckLicense(&self) -> Result<(), Error> {
		if !self.LicenseSignal.Get().await {
			return Err(Error::InvalidLicense("Invalid action license".to_string()));
		}

		Ok(())
	}

	async fn HandleDelay(&self) -> Result<(), Error> {
		if let Some(Delay) = self.Metadata.Get("Delay").await {
			let Delay = Duration::from_secs(Delay.as_u64().unwrap_or(0));

			tokio::time::sleep(Delay).await;
		}

		Ok(())
	}

	async fn ExecuteHooks(&self, Context: &Life) -> Result<(), Error> {
		if let Some(Hooks) = self.Metadata.Get("Hooks").await {
			for Hook in Hooks.as_array().unwrap_or(&Vec::new()) {
				if let Some(HookFn) = Context.Span.get(Hook.as_str().unwrap_or("")) {
					HookFn()?;
				}
			}
		}

		Ok(())
	}

	async fn ExecuteFunction(&self, ActionType: &str) -> Result<(), Error> {
		if let Some(Function) = self.Plan.Remove(ActionType) {
			self.Result(Function.borrow()(self.Argument().await?).await?).await?;
		} else {
			return Err(Error::ExecutionError(format!(
				"No function found for action type: {}",
				ActionType
			)));
		}

		Ok(())
	}

	async fn HandleNextAction(&self, Context: &Life) -> Result<(), Error> {
		if let Some(NextAction) = self.Metadata.Get("NextAction").await {
			let NextAction: Struct<T> = serde_json::from_value(NextAction.clone())
				.map_err(|e| Error::ExecutionError(format!("Failed to parse NextAction: {}", e)))?;

			NextAction.Execute(Context).await?;
		}

		Ok(())
	}

	async fn Argument(&self) -> Result<Vec<serde_json::Value>, Error> {
		Ok(vec![])
	}

	async fn Result(&self, Result: serde_json::Value) -> Result<(), Error> {
		Ok(())
	}
}

use log::info;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt::Debug, sync::Arc};

use crate::{
	Enum::Sequence::Action::Error::Enum as Error,
	Struct::Sequence::{Signal::Struct as Signal, Vector::Struct as Vector},
};

pub mod Signature;
