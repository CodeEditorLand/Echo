#![allow(non_snake_case)]

struct SimpleWorker;

#[async_trait::async_trait]
impl Worker for SimpleWorker {
	async fn Receive(
		&self,
		Action: Box<dyn Sequence::Action::Trait>,
		Context: &Life::Struct,
	) -> Result<(), Error::Enum> {
		Action.Execute(Context).await
	}
}

async fn Read(Argument: Vec<Value>) -> Result<Value, Error::Enum> {
	let mut Content = String::new();

	File::open(
		Argument[0].as_str().ok_or(Error::Enum::Execution("Invalid file path".to_string()))?,
	)
	.await
	.map_err(|e| Error::Enum::Execution(e.to_string()))?
	.read_to_string(&mut Content)
	.await
	.map_err(|e| Error::Enum::Execution(e.to_string()))?;

	Ok(json!(Content))
}

async fn Write(Argument: Vec<Value>) -> Result<Value, Error::Enum> {
	OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(Argument[0].as_str().ok_or(Error::Enum::Execution("Invalid file path".to_string()))?)
		.await
		.map_err(|e| Error::Enum::Execution(e.to_string()))?
		.write_all(
			Argument[1]
				.as_str()
				.ok_or(Error::Enum::Execution("Invalid content".to_string()))?
				.as_bytes(),
		)
		.await
		.map_err(|e| Error::Enum::Execution(e.to_string()))?;

	Ok(json!("File written successfully"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let Plan = Plan::Struct::New()
		.WithSignature(Action::Signature::Struct { Name: "Read".to_string() })
		.WithSignature(Action::Signature::Struct { Name: "Write".to_string() })
		.WithFunction("Read", Read)?
		.WithFunction("Write", Write)?
		.Build();

	let Production = Arc::new(Production::Struct::New());
	let Life = Life::Struct {
		Span: Arc::new(dashmap::DashMap::new()),
		Fate: Arc::new(config::Config::default()),
		Cache: Arc::new(tokio::sync::Mutex::new(dashmap::DashMap::new())),
		Karma: Arc::new(dashmap::DashMap::new()),
	};

	let Worker = Arc::new(SimpleWorker);
	let Sequence = Arc::new(Sequence::Struct::New(Worker, Production.clone(), Life));

	// Channel for sending action results
	let (tx, mut rx) = mpsc::unbounded_channel();

	// Spawn worker tasks
	let mut workers = JoinSet::new();
	let worker_count = 4;

	for _ in 0..worker_count {
		let sequence = Sequence.clone();
		let tx = tx.clone();

		workers.spawn(async move {
			while !sequence.Time.Get().await {
				if let Some(action) = sequence.Work.Do().await {
					let result = sequence.Worker.Receive(action, &sequence.Life).await;
					tx.send(result).unwrap();
				}
			}
		});
	}

	// Set up Tauri application
	tauri::Builder::default()
		.setup(|app| {
			let Handle = app.handle();

			// Add actions to the production line
			tokio::spawn(async move {
				Production
					.Assign(Box::new(
						Action::Struct::New(
							"Write",
							json!(["output.txt", "Hello, World!"]),
							Arc::new(Plan.clone()),
						)
						.clone(),
					))
					.await;

				Production
					.Assign(Box::new(
						Action::Struct::New("Read", json!(["input.txt"]), Arc::new(Plan.clone()))
							.clone(),
					))
					.await;

				// Process action results
				while let Some(result) = rx.recv().await {
					match result {
						Ok(_) => Handle
							.emit_all("ActionResult", "Action completed successfully")
							.unwrap(),
						Err(e) => Handle
							.emit_all("ActionResult", format!("Action failed: {}", e))
							.unwrap(),
					}
				}
			});

			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");

	// Wait for all workers to complete
	while let Some(result) = workers.join_next().await {
		if let Err(e) = result {
			eprintln!("Worker task failed: {}", e);
		}
	}

	// Shutdown the sequence
	Sequence.Shutdown().await;

	println!("Application completed");

	Ok(())
}

use serde_json::{json, Value};
use std::sync::Arc;
use tokio::{
	fs::{File, OpenOptions},
	io::{AsyncReadExt, AsyncWriteExt},
	sync::mpsc,
	task::JoinSet,
};

use Echo::{
	Enum::Sequence::Action::Error,
	Struct::Sequence::{self, Action, Life, Plan, Production},
	Trait::Sequence::Worker,
};
