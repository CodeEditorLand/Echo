use async_trait::async_trait;
use config::{Config, File};
use log::{error, info, warn};
use metrics::{counter, gauge};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
	collections::{HashMap, VecDeque},
	sync::Arc,
};
use thiserror::Error;
use tokio::{
	sync::Mutex,
	time::{sleep, Duration},
};

// Signal implementation
#[derive(Clone)]
struct Signal<T> {
	Value: Arc<Mutex<T>>,
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
	Entries: HashMap<String, Signal<serde_json::Value>>,
}

impl VectorDatabase {
	pub fn New() -> Self {
		VectorDatabase { Entries: HashMap::new() }
	}

	pub fn Insert(&mut self, Key: String, Value: serde_json::Value) {
		self.Entries.insert(Key, Signal::New(Value));
	}

	pub async fn Get(&self, Key: &str) -> Option<serde_json::Value> {
		self.Entries.get(Key).map(|signal| signal.Get().await)
	}
}

pub struct ActionSignature {
	Name: String,
	InputTypes: Vec<String>,
	OutputType: String,
}

pub struct Plan {
	Signatures: HashMap<String, ActionSignature>,
	Functions: HashMap<
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
		Self { Signatures: HashMap::new(), Functions: HashMap::new() }
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
		if let Some(Signature) = self.Signatures.get(Name) {
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
		&Box<
			dyn Fn(
					Vec<serde_json::Value>,
				)
					-> Pin<Box<dyn Future<Output = Result<serde_json::Value, ActionError>> + Send>>
				+ Send
				+ Sync,
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

#[derive(Clone)]
pub struct Action<T: Send + Sync> {
	pub Metadata: VectorDatabase,
	pub Content: T,
	pub LicenseSignal: Signal<bool>,
	pub Plan: Arc<Plan>,
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
			let Args = self.GetArgumentsFromMetadata().await?;
			let Result = Func(Args).await?;
			self.HandleFunctionResult(Result).await?;
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
	HookMap: Arc<HashMap<String, Hook>>,
	Config: Arc<Config>,
	Cache: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

type Hook = fn() -> Result<(), ActionError>;

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

// Example specific action implementation
#[derive(Clone)]
struct ReadAction {
	Path: String,
}

impl Action<ReadAction> {
	async fn ExecuteLogic(&self, _Context: &ExecutionContext) -> Result<(), ActionError> {
		info!("Reading from path: {}", self.Content.Path);
		// Implement actual read logic here
		Ok(())
	}
}

#[tokio::main]
async fn Main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let Config = Config::builder().add_source(File::with_name("config.toml")).build()?;

	let Work = Arc::new(Work::New());

	let mut HookMap = HashMap::new();

	HookMap.insert("LogStart".to_string(), || {
		info!("Action started");
		Ok(())
	} as Hook);

	HookMap.insert("Backup".to_string(), || {
		info!("Backup created");
		Ok(())
	} as Hook);

	let Context = ExecutionContext {
		HookMap: Arc::new(HookMap),
		Config: Arc::new(Config),
		Cache: Arc::new(Mutex::new(HashMap::new())),
	};

	let Plan = PlanBuilder::New()
		.WithSignature(ActionSignature {
			Name: "Read".to_string(),
			InputTypes: vec!["String".to_string()],
			OutputType: "String".to_string(),
		})
		.WithFunction("Read", |Args: Vec<serde_json::Value>| async move {
			let Path = Args[0].as_str().unwrap();
			// Implement actual read logic here
			Ok(serde_json::json!(format!("Read content from: {}", Path)))
		})?
		.Build();

	let SharedPlan = Arc::new(Plan);

	struct SimpleWorker;
	#[async_trait]
	impl Worker for SimpleWorker {
		async fn Receive(
			&self,
			Action: Box<dyn ActionTrait>,
			Context: &ExecutionContext,
		) -> Result<(), ActionError> {
			Action.Execute(Context).await
		}
	}

	let Site = Arc::new(SimpleWorker);
	let Processor = Arc::new(ActionProcessor::New(Site, Work.clone(), Context));

	let ProcessorClone = Processor.clone();
	tokio::spawn(async move { ProcessorClone.Run().await });

	let CommanderAction = Action::New("Commander", (), SharedPlan.clone())
		.WithMetadata("Role", serde_json::json!("Supervisor"));

	let ReadAction = Box::new(
		Action::New("Read", ReadAction { Path: "SomePath".to_string() }, SharedPlan.clone())
			.WithMetadata("CommandingOfficer", serde_json::to_value(&CommanderAction).unwrap())
			.WithMetadata("Hooks", serde_json::json!(["LogStart"]))
			.WithMetadata("Delay", serde_json::json!(1)),
	) as Box<dyn ActionTrait>;

	Work.Assign(ReadAction).await;

	// Wait for some time to allow actions to process
	sleep(Duration::from_secs(5)).await;

	Processor.Shutdown().await;

	Ok(())
}
