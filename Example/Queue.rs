#![allow(non_snake_case)]

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

	let mut HookMap: DashMap<String, Hook> = DashMap::new();

	HookMap.insert(
		"LogStart".to_string(),
		Arc::new(|| {
			info!("Action started");

			Ok(())
		}),
	);

	HookMap.insert(
		"Backup".to_string(),
		Arc::new(|| {
			info!("Backup created");

			Ok(())
		}) as Hook,
	);

	let Context = ExecutionContext {
		HookMap: Arc::new(HookMap),
		Config: Arc::new(Config),
		Cache: Arc::new(Mutex::new(DashMap::new())),
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

	#[derive(Serialize, Deserialize)]
	struct EmptyContent;

	let CommanderAction = Action::New("Commander", EmptyContent, SharedPlan.clone())
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
