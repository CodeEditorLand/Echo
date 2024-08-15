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
	pub fn New(Action: &str, Content: T, Plan: Arc<Formality>) -> Self {
		let mut Metadata = Vector::New();

		Metadata.Insert("Action".to_string(), serde_json::json!(Action));

		Metadata.Insert("License".to_string(), serde_json::json!("valid"));

		Struct { Metadata, Content, LicenseSignal: Signal::New(true), Plan }
	}

	pub fn WithMetadata(mut self, Key: &str, Value: serde_json::Value) -> Self {
		self.Metadata.Insert(Key.to_string(), Value);

		self
	}

	pub async fn Execute(&self, Context: &Life) -> Result<(), Error> {
		let Action = self
			.Metadata
			.Get("Action")
			.await
			.ok_or_else(|| Error::Execution("Action not found".to_string()))?
			.as_str()
			.ok_or_else(|| Error::Execution("Action is not a string".to_string()))?
			.to_string();

		info!("Executing action: {}", Action);

		self.License().await?;

		self.Delay().await?;

		self.Hooks(Context).await?;

		self.Function(&Action).await?;

		self.Next(Context).await?;

		Ok(())
	}

	async fn License(&self) -> Result<(), Error> {
		if !self.LicenseSignal.Get().await {
			return Err(Error::License("Invalid action license".to_string()));
		}

		Ok(())
	}

	async fn Delay(&self) -> Result<(), Error> {
		if let Some(Delay) = self.Metadata.Get("Delay").await {
			let Delay = Duration::from_secs(Delay.as_u64().unwrap_or(0));

			tokio::time::sleep(Delay).await;
		}

		Ok(())
	}

	async fn Hooks(&self, Context: &Life) -> Result<(), Error> {
		if let Some(Hooks) = self.Metadata.Get("Hooks").await {
			for Hook in Hooks.as_array().unwrap_or(&Vec::new()) {
				if let Some(HookFn) = Context.Span.get(Hook.as_str().unwrap_or("")) {
					HookFn()?;
				}
			}
		}

		Ok(())
	}

	async fn Function(&self, Action: &str) -> Result<(), Error> {
		if let Some(Function) = self.Plan.Remove(Action) {
			self.Result(Function.borrow()(self.Argument().await?).await?).await?;
		} else {
			return Err(Error::Execution(format!(
				"No function found for action type: {}",
				Action
			)));
		}

		Ok(())
	}

	async fn Next(&self, Context: &Life) -> Result<(), Error> {
		if let Some(NextAction) = self.Metadata.Get("NextAction").await {
			let NextAction: Struct<T> = serde_json::from_value(NextAction.clone())
				.map_err(|e| Error::Execution(format!("Failed to parse NextAction: {}", e)))?;

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
