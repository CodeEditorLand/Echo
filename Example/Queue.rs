#![allow(non_snake_case)]

#[async_trait]
pub trait ActionLogic {
	type Content;
	async fn Execute(&self, Context: &ExecutionContext) -> Result<Self::Content, ActionError>;
}

// Example specific action implementations
#[derive(Clone, Serialize, Deserialize)]
struct ReadAction {
	pub Path: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ProcessQueueAction {
	QueueName: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct PrintAction {
	Content: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct FilePrintAction {
	pub ReadAction: ReadAction,
	pub OutputPath: String,
}

impl ActionLogic for Action<ReadAction> {
	type Content = String;

	async fn Execute(&self, _Context: &ExecutionContext) -> Result<String, ActionError> {
		info!("Reading from path: {}", self.Content.Path);

		let mut Content = String::new();

		File_tokio::open(&self.Content.Path)
			.await
			.map_err(|e| ActionError::ExecutionError(format!("Failed to open file: {}", e)))?
			.read_to_string(&mut Content)
			.await
			.map_err(|e| ActionError::ExecutionError(format!("Failed to read file: {}", e)))?;

		Ok(Content)
	}
}

impl ActionLogic for Action<ProcessQueueAction> {
	type Content = ();
	async fn Execute(&self, Context: &ExecutionContext) -> Result<(), ActionError> {
		info!("Processing queue: {}", self.Content.QueueName);

		let Queue = Context.Queues.get(&self.Content.QueueName).ok_or_else(|| {
			ActionError::ExecutionError(format!("Queue {} not found", self.Content.QueueName))
		})?;

		while let Some(Action) = Queue.Execute().await {
			Action.Execute(Context).await?;
		}

		Ok(())
	}
}

impl ActionLogic for Action<PrintAction> {
	type Content = ();
	async fn Execute(&self, _Context: &ExecutionContext) -> Result<(), ActionError> {
		println!("Printing content: {}", self.Content.Content);

		Ok(())
	}
}

impl ActionLogic for Action<FilePrintAction> {
	type Content = ();
	async fn Execute(&self, Context: &ExecutionContext) -> Result<(), ActionError> {
		// Execute the ReadAction to get the content
		File_tokio::create(&self.Content.OutputPath)
			.await
			.map_err(|e| ActionError::ExecutionError(format!("Failed to create file: {}", e)))?
			.write_all(
				Action::New("Read", self.Content.ReadAction.clone(), self.Plan.clone())
					.Execute(Context)
					.await?
					.into(),
			)
			.await
			.map_err(|e| ActionError::ExecutionError(format!("Failed to write to file: {}", e)))?;

		info!("Content written to file: {}", self.Content.OutputPath);

		Ok(())
	}
}

struct Worker;

#[async_trait]
impl Worker for Worker {
	async fn Receive(
		&self,
		Action: Box<dyn ActionTrait>,
		Context: &ExecutionContext,
	) -> Result<(), ActionError> {
		Action.Execute(Context).await
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let Config = Config::builder().add_source(File_config::with_name("config.toml")).build()?;

	let MainWork = Arc::new(Work::New());

	let SecondaryWork = Arc::new(Work::New());

	let mut HookMap: DashMap<String, Hook> = DashMap::new();

	HookMap.insert(
		"LogStart".to_string(),
		Arc::new(|| {
			info!("Action started");

			Ok(())
		}) as Hook,
	);

	let mut Queues = DashMap::new();

	Queues.insert("Main".to_string(), MainWork.clone());

	Queues.insert("Secondary".to_string(), SecondaryWork.clone());

	let Context = ExecutionContext {
		HookMap: Arc::new(HookMap),
		Config: Arc::new(Config),
		Cache: Arc::new(tokio::sync::Mutex::new(DashMap::new())),
		Queues: Arc::new(Queues),
	};

	let Plan = PlanBuilder::New()
		.WithSignature(ActionSignature {
			Name: "Read".to_string(),
			InputTypes: vec!["String".to_string()],
			OutputType: "String".to_string(),
		})
		.WithFunction("Read", |Args: Vec<serde_json::Value>| async move {
			let Path = Args[0].as_str().unwrap();

			let mut Content = String::new();

			File_tokio::open(Path)
				.await
				.map_err(|e| ActionError::ExecutionError(format!("Failed to open file: {}", e)))?
				.read_to_string(&mut Content)
				.await
				.map_err(|e| ActionError::ExecutionError(format!("Failed to read file: {}", e)))?;

			Ok(serde_json::json!(Content))
		})?
		.WithSignature(ActionSignature {
			Name: "ProcessQueue".to_string(),
			InputTypes: vec!["String".to_string()],
			OutputType: "()".to_string(),
		})
		.WithFunction("ProcessQueue", |Args: Vec<serde_json::Value>| async move {
			Ok(serde_json::json!(format!("Processed queue: {}", Args[0].as_str().unwrap())))
		})?
		.WithSignature(ActionSignature {
			Name: "Print".to_string(),
			InputTypes: vec!["String".to_string()],
			OutputType: "()".to_string(),
		})
		.WithFunction("Print", |Args: Vec<serde_json::Value>| async move {
			let Content = Args[0].as_str().unwrap();

			println!("Printing content: {}", Content);

			Ok(serde_json::json!(null))
		})?
		.WithSignature(ActionSignature {
			Name: "FilePrint".to_string(),
			InputTypes: vec!["String".to_string(), "String".to_string()],
			OutputType: "()".to_string(),
		})
		.WithFunction("FilePrint", |Args: Vec<serde_json::Value>| async move {
			let Content = Args[0].as_str().unwrap();

			let OutputPath = Args[1].as_str().unwrap();

			tokio::fs::File_tokio(OutputPath)
				.await
				.map_err(|e| ActionError::ExecutionError(format!("Failed to create file: {}", e)))?
				.write_all(Content.as_bytes())
				.await
				.map_err(|e| {
					ActionError::ExecutionError(format!("Failed to write to file: {}", e))
				})?;

			info!("Content written to file: {}", OutputPath);

			Ok(serde_json::json!(null))
		})?
		.Build();

	let SharedPlan = Arc::new(Plan);

	let Site = Arc::new(Worker);

	let Processor = Arc::new(ActionProcessor::New(Site, MainWork.clone(), Context.clone()));

	let ProcessorClone = Processor.clone();

	tokio::spawn(async move { ProcessorClone.Run().await });

	#[derive(Clone, Serialize, Deserialize)]
	struct EmptyContent;

	let CommanderAction = Action::New("Commander", EmptyContent, SharedPlan.clone())
		.WithMetadata("Role", serde_json::json!("Supervisor"));

	// Add actions to the main queue
	MainWork
		.Assign(Box::new(
			Action::New("Read", ReadAction { Path: "SomePath".to_string() }, SharedPlan.clone())
				.WithMetadata("CommandingOfficer", serde_json::to_value(&CommanderAction).unwrap())
				.WithMetadata("Hooks", serde_json::json!(["LogStart"]))
				.WithMetadata("Delay", serde_json::json!(1)),
		) as Box<dyn ActionTrait>)
		.await;

	MainWork
		.Assign(Box::new(
			Action::New("Print", PrintAction { Content: "".to_string() }, SharedPlan.clone())
				.WithMetadata("CommandingOfficer", serde_json::to_value(&CommanderAction).unwrap())
				.WithMetadata("Hooks", serde_json::json!(["LogStart"])),
		) as Box<dyn ActionTrait>)
		.await;

	MainWork
		.Assign(Box::new(
			Action::New(
				"FilePrint",
				FilePrintAction {
					ReadAction: ReadAction { Path: "input.txt".to_string() },
					OutputPath: "output.txt".to_string(),
				},
				SharedPlan.clone(),
			)
			.WithMetadata("CommandingOfficer", serde_json::to_value(&CommanderAction).unwrap())
			.WithMetadata("Hooks", serde_json::json!(["LogStart"])),
		) as Box<dyn ActionTrait>)
		.await;

	MainWork
		.Assign(Box::new(
			Action::New(
				"ProcessQueue",
				ProcessQueueAction { QueueName: "Secondary".to_string() },
				SharedPlan.clone(),
			)
			.WithMetadata("CommandingOfficer", serde_json::to_value(&CommanderAction).unwrap())
			.WithMetadata("Hooks", serde_json::json!(["LogStart"])),
		) as Box<dyn ActionTrait>)
		.await;

	// Add some actions to the secondary queue
	SecondaryWork
		.Assign(Box::new(
			Action::New(
				"Print",
				PrintAction { Content: format!("This is from the secondary queue") },
				SharedPlan.clone(),
			)
			.WithMetadata("CommandingOfficer", serde_json::to_value(&CommanderAction).unwrap())
			.WithMetadata("Hooks", serde_json::json!(["LogStart"])),
		) as Box<dyn ActionTrait>)
		.await;

	// Wait for some time to allow actions to process
	sleep(Duration::from_secs(10)).await;

	Processor.Shutdown().await;

	Ok(())
}

use async_trait::async_trait;
use config::{Config, File as File_config, FileFormat as FileFormat_config};
use dashmap::DashMap;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{
	fs::File as File_tokio,
	io::{AsyncReadExt, AsyncWriteExt},
	time::{sleep, Duration},
};

use Echo::Queue::*;
