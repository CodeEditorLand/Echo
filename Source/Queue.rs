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

	pub async fn Set(&self, value: T) {
		*self.0.lock().await = value;
	}
}

// Isolate VectorDatabase logic from Action
#[derive(Clone, Debug)]
pub struct VectorDatabase {
	Entries: DashMap<String, serde_json::Value>,
}

impl VectorDatabase {
	pub fn New() -> Self {
		Self { Entries: DashMap::new() }
	}

	pub fn Insert(&mut self, Key: String, Value: serde_json::Value) {
		self.Entries.insert(Key, Value);
	}

	pub async fn Get(&self, Key: &str) -> Option<serde_json::Value> {
		self.Entries.get(Key).map(|v| v.value().clone())
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
	name: String,
}

#[derive(Debug)]
pub struct Plan {
	Signatures: DashMap<String, ActionSignature>,
	Functions: DashMap<
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

impl Plan {
	pub fn New() -> Self {
		Self { Signatures: DashMap::new(), Functions: DashMap::new() }
	}

	pub fn AddSignature(&mut self, Signature: ActionSignature) -> &mut Self {
		self.Signatures.insert(Signature.Name.clone(), Signature);

		self
	}

	pub fn AddFunction<F, Fut>(&mut self, Name: &str, Function: F) -> Result<&mut Self, String>
	where
		F: Fn(Vec<serde_json::Value>) -> Fut + Send + Sync + 'static,
		Fut: Future<Output = Result<serde_json::Value, ActionError>> + Send + 'static,
	{
		// Return early if no signature
		if !self.Signatures.contains_key(Name) {
			return Err(format!("No signature found for function: {}", Name));
		}

		let BoxedFunc = Box::new(move |Args: Vec<serde_json::Value>| {
			Box::pin(Function(Args))
				as Pin<Box<dyn Future<Output = Result<serde_json::Value, ActionError>> + Send>>
		});

		self.Functions.insert(Name.to_string(), BoxedFunc);

		Ok(self)
	}

	pub fn GetFunction(
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
		self.Functions.get(Name)
	}
}

pub struct PlanBuilder {
	Plan: Plan,
}

impl PlanBuilder {
	pub fn New() -> Self {
		Self { Plan: Plan::New() }
	}

	pub fn WithSignature(mut self, Signature: ActionSignature) -> Self {
		self.Plan.AddSignature(Signature);

		self
	}

	pub fn WithFunction<F, Fut>(mut self, Name: &str, Function: F) -> Result<Self, String>
	where
		F: Fn(Vec<serde_json::Value>) -> Fut + Send + Sync + 'static,
		Fut: Future<Output = Result<serde_json::Value, ActionError>> + Send + 'static,
	{
		self.Plan.AddFunction(Name, Function)?;

		Ok(self)
	}

	pub fn Build(self) -> Plan {
		self.Plan
	}
}

#[derive(Clone, Debug)]
pub struct Action<T: Send + Sync> {
	pub Metadata: VectorDatabase,
	pub Content: T,
	pub LicenseSignal: Signal<bool>,
	pub Plan: Arc<Plan>,
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
	pub fn New(ActionType: &str, Content: T, Plan: Arc<Plan>) -> Self {
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

	pub async fn Execute(&self, Context: &ExecutionContext) -> Result<(), ActionError> {
		// Can we avoid this unwrap chain?
		let ActionType =
			self.Metadata.Get("ActionType").await.unwrap().as_str().unwrap().to_string();

		info!("Executing action: {}", ActionType);

		// These checks can be extracted to separate functions for clarity
		if let Some(Officer) = self.Metadata.Get("CommandingOfficer").await {
			if Officer.get("License").unwrap().as_str().unwrap() != "valid" {
				return Err(ActionError::InvalidLicense(
					"Invalid commanding officer license".to_string(),
				));
			}
		}

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
				if let Some(HookFn) = Context.HookMap.get(Hook.as_str().unwrap()) {
					HookFn()?;
				}
			}
		}

		// This could be simplified if `Action` new up the function on creation
		if let Some(Function) = self.Plan.GetFunction(&ActionType) {
			let Args = self.GetArgumentsFromMetadata().await?;

			let Result = Function.borrow()(Args).await?;

			self.HandleFunctionResult(Result).await?;
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
	async fn GetArgumentsFromMetadata(&self) -> Result<Vec<serde_json::Value>, ActionError> {
		Ok(vec![])
	}

	async fn HandleFunctionResult(&self, Result: serde_json::Value) -> Result<(), ActionError> {
		Ok(())
	}
}

#[async_trait]
pub trait ActionTrait: Send + Sync {
	async fn Execute(&self, Context: &ExecutionContext) -> Result<(), ActionError>;

	fn CloneAction(&self) -> Box<dyn ActionTrait>;
}

#[async_trait]
impl<T: Send + Sync + Clone + 'static> ActionTrait for Action<T> {
	async fn Execute(&self, Context: &ExecutionContext) -> Result<(), ActionError> {
		self.Execute(Context).await
	}

	fn CloneAction(&self) -> Box<dyn ActionTrait> {
		Box::new(self.clone())
	}
}

pub struct ExecutionContext {
	pub HookMap: Arc<DashMap<String, Hook>>,
	pub Config: Arc<Config>,
	Cache: Arc<Mutex<DashMap<String, serde_json::Value>>>,
	pub Queues: Arc<DashMap<String, Arc<Work>>>,
}

type Hook = Arc<dyn Fn() -> Result<(), ActionError> + Send + Sync>;

#[async_trait]
pub trait Worker: Send + Sync {
	async fn Receive(
		&self,
		Action: Box<dyn ActionTrait>,
		Context: &ExecutionContext,
	) -> Result<(), ActionError>;
}

pub struct Work {
	Queue: Arc<Mutex<VecDeque<Box<dyn ActionTrait>>>>,
}

impl Work {
	pub fn New() -> Self {
		Work { Queue: Arc::new(Mutex::new(VecDeque::new())) }
	}

	pub async fn Execute(&self) -> Option<Box<dyn ActionTrait>> {
		self.Queue.lock().await.pop_front()
	}

	pub async fn Assign(&self, Action: Box<dyn ActionTrait>) {
		self.Queue.lock().await.push_back(Action);
	}
}

pub struct ActionProcessor {
	Site: Arc<dyn Worker>,
	Work: Arc<Work>,
	Context: ExecutionContext,
	ShutdownSignal: Signal<bool>,
}

impl ActionProcessor {
	pub fn New(Site: Arc<dyn Worker>, Work: Arc<Work>, Context: ExecutionContext) -> Self {
		ActionProcessor { Site, Work, Context, ShutdownSignal: Signal::New(false) }
	}

	pub async fn Run(&self) {
		while !self.ShutdownSignal.Get().await {
			if let Some(Action) = self.Work.Execute().await {
				let Result = self.ExecuteWithRetry(Action).await;

				if let Err(e) = Result {
					error!("Error processing action: {}", e);
				}
			}
		}
	}

	async fn ExecuteWithRetry(&self, Action: Box<dyn ActionTrait>) -> Result<(), ActionError> {
		// Make this configurable?
		let MaxRetries = self.Context.Config.get_int("max_retries").unwrap_or(3) as u32;

		let mut Retries = 0;

		loop {
			match self.Site.Receive(Action.CloneAction(), &self.Context).await {
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
		self.ShutdownSignal.Set(true).await;
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
use tokio::{sync::Mutex, time::sleep};
