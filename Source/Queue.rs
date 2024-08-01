use async_trait::async_trait;
use config::{Config, File};
use dashmap::DashMap;
use futures::Future;
use log::{error, info, warn};
use metrics::{counter, gauge};
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{borrow::Borrow, collections::VecDeque, pin::Pin, sync::Arc};
use thiserror::Error;
use tokio::{
	sync::Mutex,
	time::{sleep, Duration},
};

#[derive(Clone, Debug)]
struct Signal<T> {
	Value: Arc<Mutex<T>>,
}

impl<T: Serialize> Serialize for Signal<T> {
	async fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let guard = self.Value.lock().await;
		T::serialize(&*guard, serializer)
	}
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Signal<T> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let value = T::deserialize(deserializer)?;
		Ok(Signal { Value: Arc::new(Mutex::new(value)) })
	}
}

impl<T: Clone> Signal<T> {
	fn New(InitialValue: T) -> Self {
		Signal { Value: Arc::new(Mutex::new(InitialValue)) }
	}

	async fn Get(&self) -> T {
		self.Value.lock().await.clone()
	}

	async fn Set(&self, NewValue: T) {
		*self.Value.lock().await = NewValue;
	}
}

#[derive(Error, Debug)]
pub enum ActionError {
	#[error("Invalid license: {0}")]
	InvalidLicense(String),
	#[error("Execution error: {0}")]
	ExecutionError(String),
	#[error("Routing error: {0}")]
	RoutingError(String),
	#[error("Cancellation error: {0}")]
	CancellationError(String),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VectorDatabase {
	Entries: DashMap<String, Signal<serde_json::Value>>,
}

impl VectorDatabase {
	pub fn New() -> Self {
		VectorDatabase { Entries: DashMap::new() }
	}

	pub fn Insert(&mut self, Key: String, Value: serde_json::Value) {
		self.Entries.insert(Key, Signal::New(Value));
	}

	pub async fn Get(&self, Key: &str) -> Option<serde_json::Value> {
		self.Entries.get(Key).map(|signal| signal.Get().await)
	}
}

#[derive(Debug)]
pub struct ActionSignature {
	Name: String,
	InputTypes: Vec<String>,
	OutputType: String,
}

struct DebugWrapper<T>(T);

impl<T> fmt::Debug for DebugWrapper<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Function")
	}
}

#[derive(Debug)]
pub struct Plan {
	Signatures: DashMap<String, ActionSignature>,
	Functions: DashMap<
		String,
		Box<
			dyn Fn(Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, ActionError>> + Send>>
				+ Send
				+ Sync,
		>,
	>,
}

use std::fmt;

impl fmt::Debug for Plan {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Plan")
			.field("Signatures", &self.Signatures)
			// TODO: IMPLEMENT DEBUG
			// .field("Functions", &self.Functions)
			.finish()
	}
}

impl Plan {
	pub fn New() -> Self {
		Self { Signatures: DashMap::new(), Functions: DashMap::new() }
	}

	pub fn AddSignature(&mut self, Signature: ActionSignature) -> &mut Self {
		self.Signatures.insert(Signature.Name.clone(), Signature);

		self
	}

	pub fn AddFunction<F, Fut>(&mut self, Name: &str, Func: F) -> Result<&mut Self, String>
	where
		F: Fn(Vec<serde_json::Value>) -> Fut + Send + Sync + 'static,
		Fut: Future<Output = Result<serde_json::Value, ActionError>> + Send + 'static,
	{
		if let Some(_Signature) = self.Signatures.get(Name) {
			let BoxedFunc = Box::new(move |Args: Vec<serde_json::Value>| {
				Box::pin(Func(Args))
					as Pin<Box<dyn Future<Output = Result<serde_json::Value, ActionError>> + Send>>
			});

			self.Functions.insert(Name.to_string(), BoxedFunc);

			Ok(self)
		} else {
			Err(format!("No signature found for function: {}", Name))
		}
	}

	pub fn GetFunction(
		&self,
		Name: &str,
	) -> Option<
		impl Borrow<
			Box<
				dyn Fn(
						Vec<Value>,
					) -> Pin<Box<dyn Future<Output = Result<Value, ActionError>> + Send>>
					+ Send
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

	pub fn WithFunction<F, Fut>(mut self, Name: &str, Func: F) -> Result<Self, String>
	where
		F: Fn(Vec<serde_json::Value>) -> Fut + Send + Sync + 'static,
		Fut: Future<Output = Result<serde_json::Value, ActionError>> + Send + 'static,
	{
		self.Plan.AddFunction(Name, Func)?;

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
		// Implement serialization logic
		// You may need to create a custom struct to hold serializable data
		unimplemented!()
	}
}

impl<'de, T: Send + Sync + Deserialize<'de>> Deserialize<'de> for Action<T> {
	fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		// Implement deserialization logic
		// You may need to create a custom struct to hold deserializable data
		unimplemented!()
	}
}

impl<T: Send + Sync> Action<T> {
	pub fn New(ActionType: &str, Content: T, Plan: Arc<Plan>) -> Self {
		let mut Metadata = VectorDatabase::New();

		Metadata.Insert("ActionType".to_string(), serde_json::json!(ActionType));

		Metadata.Insert("License".to_string(), serde_json::json!("valid"));

		Action { Metadata, Content, LicenseSignal: Signal::New(true), Plan }
	}

	pub fn WithMetadata(mut self, Key: &str, Value: serde_json::Value) -> Self {
		self.Metadata.Insert(Key.to_string(), Value);

		self
	}

	pub async fn Execute(&self, Context: &ExecutionContext) -> Result<(), ActionError> {
		let ActionType =
			self.Metadata.Get("ActionType").await.unwrap().as_str().unwrap().to_string();

		info!("Executing action: {}", ActionType);

		// Check licenses
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

		// Apply delay if specified
		if let Some(Delay) = self.Metadata.Get("Delay").await {
			let Delay = Duration::from_secs(Delay.as_u64().unwrap());

			sleep(Delay).await;
		}

		// Execute hooks
		if let Some(Hooks) = self.Metadata.Get("Hooks").await {
			for Hook in Hooks.as_array().unwrap() {
				if let Some(HookFn) = Context.HookMap.get(Hook.as_str().unwrap()) {
					HookFn()?;
				}
			}
		}

		// Execute action-specific logic using the Plan
		if let Some(Func) = self.Plan.GetFunction(&ActionType) {
			// let Args = self.GetArgumentsFromMetadata().await?;

			// let Result = Func(Args).await?;

			// self.HandleFunctionResult(Result).await?;

			if let Some(func_borrow) = self.Plan.GetFunction(&ActionType) {
				let func = func_borrow.borrow();
				let Args = self.GetArgumentsFromMetadata().await?;
				let Result = func(Args).await?;
				self.HandleFunctionResult(Result).await?;
			} else {
				return Err(ActionError::ExecutionError(format!(
					"No function found for action type: {}",
					ActionType
				)));
			}
		} else {
			return Err(ActionError::ExecutionError(format!(
				"No function found for action type: {}",
				ActionType
			)));
		}

		// Execute next action if exists
		if let Some(NextAction) = self.Metadata.Get("NextAction").await {
			let NextAction: Action<T> = serde_json::from_value(NextAction.clone()).unwrap();

			NextAction.Execute(Context).await?;
		}

		Ok(())
	}

	async fn GetArgumentsFromMetadata(&self) -> Result<Vec<serde_json::Value>, ActionError> {
		// Implementation to extract arguments from Metadata
		// This is a placeholder and should be implemented based on your specific requirements
		Ok(vec![])
	}

	async fn HandleFunctionResult(&self, Result: serde_json::Value) -> Result<(), ActionError> {
		// Implementation to handle the result of the function execution
		// This is a placeholder and should be implemented based on your specific requirements
		Ok(())
	}
}

pub struct ExecutionContext {
	HookMap: Arc<DashMap<String, Hook>>,
	Config: Arc<Config>,
	Cache: Arc<Mutex<DashMap<String, serde_json::Value>>>,
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
		let MaxRetries = self.Context.Config.get_int("max_retries").unwrap_or(3) as u32;

		let mut Retries = 0;

		loop {
			match self.Site.Receive(Action.Clone(), &self.Context).await {
				Ok(_) => return Ok(()),
				Err(e) => {
					if Retries >= MaxRetries {
						return Err(e);
					}
					Retries += 1;

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

#[async_trait]
pub trait ActionTrait: Send + Sync {
	async fn Execute(&self, Context: &ExecutionContext) -> Result<(), ActionError>;
	fn Clone(&self) -> Box<dyn ActionTrait>;
}

#[async_trait]
impl<T: Send + Sync + Clone + 'static> ActionTrait for Action<T> {
	async fn Execute(&self, Context: &ExecutionContext) -> Result<(), ActionError> {
		self.Execute(Context).await
	}

	fn Clone(&self) -> Box<dyn ActionTrait> {
		Box::new(self.clone())
	}
}
