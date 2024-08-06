// This should be a separate struct
#[derive(Clone, Debug)]
pub struct Signal<T>(Arc<Mutex<T>>);

impl<T> Signal<T> {
	pub fn New(value: T) -> Self {
		Signal(Arc::new(Mutex::new(value)))
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

// Isolate VectorDatabase logic from Action
#[derive(Clone, Debug)]
pub struct VectorDatabase {
	Entry: DashMap<String, serde_json::Value>,
}

impl VectorDatabase {
	pub fn New() -> Self {
		Self { Entry: DashMap::new() }
	}

	pub fn Insert(&mut self, Key: String, Value: serde_json::Value) {
		self.Entry.insert(Key, Value);
	}

	pub async fn Get(&self, Key: &str) -> Option<serde_json::Value> {
		self.Entry.get(Key).map(|v| v.value().clone())
	}
}

#[derive(Debug, Error)]
pub enum ActionError {
	#[error("Invalid License: {0}")]
	InvalidLicense(String),
	#[error("Execution Error: {0}")]
	ExecutionError(String),
}

// Placeholder for ActionSignature
#[derive(Clone, Debug)]
pub struct ActionSignature {
	Name: String,
}

#[derive(Debug)]
pub struct Formality {
	Signature: DashMap<String, ActionSignature>,
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

impl Formality {
	pub fn New() -> Self {
		Self { Signature: DashMap::new(), Function: DashMap::new() }
	}

	pub fn AddSignature(&mut self, Signature: ActionSignature) -> &mut Self {
		self.Signature.insert(Signature.Name.clone(), Signature);

		self
	}

	pub fn FunctionA<F, Fut>(&mut self, Name: &str, Function: F) -> Result<&mut Self, String>
	where
		F: Fn(Vec<serde_json::Value>) -> Fut + Send + Sync + 'static,
		Fut: Future<Output = Result<serde_json::Value, ActionError>> + Send + 'static,
	{
		// Return early if no signature
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

	pub fn FunctionB(
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

pub struct Plan {
	Formality: Formality,
}

impl Plan {
	pub fn New() -> Self {
		Self { Formality: Formality::New() }
	}

	pub fn WithSignature(mut self, Signature: ActionSignature) -> Self {
		self.Formality.AddSignature(Signature);

		self
	}

	pub fn WithFunction<F, Fut>(mut self, Name: &str, Function: F) -> Result<Self, String>
	where
		F: Fn(Vec<serde_json::Value>) -> Fut + Send + Sync + 'static,
		Fut: Future<Output = Result<serde_json::Value, ActionError>> + Send + 'static,
	{
		self.Formality.FunctionA(Name, Function)?;

		Ok(self)
	}

	pub fn Build(self) -> Formality {
		self.Formality
	}
}

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

		// Can these be statically typed?
		Metadata.Insert("ActionType".to_string(), serde_json::json!(ActionType));

		Metadata.Insert("License".to_string(), serde_json::json!("valid"));

		Action { Metadata, Content, LicenseSignal: Signal::New(true), Plan }
	}

	pub fn WithMetadata(mut self, Key: &str, Value: serde_json::Value) -> Self {
		self.Metadata.Insert(Key.to_string(), Value);

		self
	}

	pub async fn Execute(&self, Context: &Life) -> Result<(), ActionError> {
		// Can we avoid this unwrap chain?
		let ActionType =
			self.Metadata.Get("ActionType").await.unwrap().as_str().unwrap().to_string();

		info!("Executing action: {}", ActionType);

		// These checks can be extracted to separate functions for clarity
		if !self.LicenseSignal.Get().await {
			return Err(ActionError::InvalidLicense("Invalid action license".to_string()));
		}

		if let Some(Delay) = self.Metadata.Get("Delay").await {
			let Delay = Duration::from_secs(Delay.as_u64().unwrap());

			sleep(Delay).await;
		}

		// Consider using an enum or similar for different hook types
		if let Some(Hooks) = self.Metadata.Get("Hooks").await {
			for Hook in Hooks.as_array().unwrap() {
				if let Some(HookFn) = Context.Span.get(Hook.as_str().unwrap()) {
					HookFn()?;
				}
			}
		}

		// This could be simplified if `Action` new up the function on creation
		if let Some(Function) = self.Plan.FunctionB(&ActionType) {
			let Args = self.Argument().await?;

			let Result = Function.borrow()(Args).await?;

			self.Result(Result).await?;
		} else {
			return Err(ActionError::ExecutionError(format!(
				"No function found for action type: {}",
				ActionType
			)));
		}

		if let Some(NextAction) = self.Metadata.Get("NextAction").await {
			// Can this be done without cloning and unwrapping?
			let NextAction: Action<T> = serde_json::from_value(NextAction.clone()).unwrap();

			NextAction.Execute(Context).await?;
		}

		Ok(())
	}

	// Is it possible to type these to return specific values?
	async fn Argument(&self) -> Result<Vec<serde_json::Value>, ActionError> {
		Ok(vec![])
	}

	async fn Result(&self, Result: serde_json::Value) -> Result<(), ActionError> {
		Ok(())
	}
}

#[async_trait]
pub trait ActionTrait: Send + Sync {
	async fn Execute(&self, Context: &Life) -> Result<(), ActionError>;

	fn Clone(&self) -> Box<dyn ActionTrait>;
}

#[async_trait]
impl<T: Send + Sync + Clone + 'static> ActionTrait for Action<T> {
	async fn Execute(&self, Context: &Life) -> Result<(), ActionError> {
		self.Execute(Context).await
	}

	fn Clone(&self) -> Box<dyn ActionTrait> {
		Box::new(self.clone())
	}
}

pub struct Life {
	pub Span: Arc<DashMap<String, StartEnd>>,
	pub Fate: Arc<Config>,
	Cache: Arc<Mutex<DashMap<String, serde_json::Value>>>,
	pub Karma: Arc<DashMap<String, Arc<Production>>>,
}

type StartEnd = Arc<dyn Fn() -> Result<(), ActionError> + Send + Sync>;

#[async_trait]
pub trait Worker: Send + Sync {
	async fn Receive(
		&self,
		Action: Box<dyn ActionTrait>,
		Context: &Life,
	) -> Result<(), ActionError>;
}

pub struct Production {
	Line: Arc<Mutex<VecDeque<Box<dyn ActionTrait>>>>,
}

impl Production {
	pub fn New() -> Self {
		Production { Line: Arc::new(Mutex::new(VecDeque::new())) }
	}

	pub async fn Do(&self) -> Option<Box<dyn ActionTrait>> {
		self.Line.lock().await.pop_front()
	}

	pub async fn Take(&self, Action: Box<dyn ActionTrait>) {
		self.Line.lock().await.push_back(Action);
	}
}

pub struct Sequence {
	Site: Arc<dyn Worker>,
	Work: Arc<Production>,
	Life: Life,
	Time: Signal<bool>,
}

impl Sequence {
	pub fn New(Site: Arc<dyn Worker>, Work: Arc<Production>, Context: Life) -> Self {
		Sequence { Site, Work, Life: Context, Time: Signal::New(false) }
	}

	pub async fn Run(&self) {
		while !self.Time.Get().await {
			if let Some(Action) = self.Work.Do().await {
				let Result = self.ExecuteWithRetry(Action).await;

				if let Err(e) = Result {
					error!("Error processing action: {}", e);
				}
			}
		}
	}

	async fn ExecuteWithRetry(&self, Action: Box<dyn ActionTrait>) -> Result<(), ActionError> {
		// Make this configurable?
		let MaxRetries = self.Life.Fate.get_int("max_retries").unwrap_or(3) as u32;

		let mut Retries = 0;

		loop {
			match self.Site.Receive(Action.Clone(), &self.Life).await {
				Ok(_) => return Ok(()),
				Err(e) => {
					if Retries >= MaxRetries {
						return Err(e);
					}

					Retries += 1;

					// Can we make this backoff strategy configurable?
					let Delay = Duration::from_secs(
						2u64.pow(Retries) + rand::thread_rng().gen_range(0..1000),
					);

					warn!(
						"Action failed, retrying in {:?}. Attempt {} of {}",
						Delay, Retries, MaxRetries
					);

					sleep(Delay).await;
				}
			}
		}
	}

	pub async fn Shutdown(&self) {
		self.Time.Set(true).await;
	}
}

use async_trait::async_trait;
use config::{Config, File};
use dashmap::DashMap;
use futures::Future;
use log::{error, info, warn};
use metrics::{counter, gauge};
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{borrow::Borrow, collections::VecDeque, fmt::Debug, pin::Pin, sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{
	sync::Mutex,
	time::{sleep, Duration},
};
