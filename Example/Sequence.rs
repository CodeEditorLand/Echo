#![allow(non_snake_case)]

// Define a simple worker that implements the Worker trait
struct SimpleWorker;

#[async_trait::async_trait]
impl Worker for SimpleWorker {
	async fn Receive(
		&self,
		Action: Box<dyn Echo::Trait::Sequence::Action::Trait>,
		Context: &Life,
	) -> Result<(), Error> {
		Action.Execute(Context).await
	}
}

// Define actions for file reading and writing
async fn Read(Argument: Vec<Value>) -> Result<Value, Error> {
	let mut Content = String::new();

	File::open(Argument[0].as_str().ok_or(Error::Execution("Invalid file path".to_string()))?)
		.await
		.map_err(|e| Error::Execution(e.to_string()))?
		.read_to_string(&mut Content)
		.await
		.map_err(|e| Error::Execution(e.to_string()))?;

	Ok(json!(Content))
}

async fn Write(args: Vec<Value>) -> Result<Value, Error> {
	OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(args[0].as_str().ok_or(Error::Execution("Invalid file path".to_string()))?)
		.await
		.map_err(|e| Error::Execution(e.to_string()))?
		.write_all(
			args[1].as_str().ok_or(Error::Execution("Invalid content".to_string()))?.as_bytes(),
		)
		.await
		.map_err(|e| Error::Execution(e.to_string()))?;

	Ok(json!("File written successfully"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Create a plan with file reading and writing actions
	let Plan = Arc::new(
		Echo::Struct::Sequence::Plan::Struct::New()
			.WithSignature(Signature { Name: "Read".to_string() })
			.WithSignature(Signature { Name: "Write".to_string() })
			.WithFunction("Read", Read)?
			.WithFunction("Write", Write)?
			.Build(),
	);

	// Create a production line
	let Production = Arc::new(Echo::Struct::Sequence::Production::Struct::New());

	// Create a life context
	let Life = Life {
		Span: Arc::new(dashmap::DashMap::new()),
		Fate: Arc::new(config::Config::default()),
		Cache: Arc::new(tokio::sync::Mutex::new(dashmap::DashMap::new())),
		Karma: Arc::new(dashmap::DashMap::new()),
	};

	// Create a worker
	let Worker = Arc::new(SimpleWorker);

	// Create a sequence
	let Sequence = Echo::Struct::Sequence::Struct::New(Worker, Production.clone(), Life);

	// Add actions to the production line

	// Create actions for reading and writing files
	Production
		.Assign(Box::new(
			Action::New("Write", json!(["output.txt", "Hello, World!"]), Plan.clone()).clone(),
		))
		.await;

	Production
		.Assign(Box::new(Action::New("Read", json!(["input.txt"]), Plan.clone()).clone()))
		.await;

	let CloneSequence = Sequence.clone();

	// Run the sequence
	tokio::spawn(async move {
		// TODO: thread 'tokio-runtime-worker' has overflowed its stack
		// CloneSequence.Run().await;
	});

	// Wait for a moment to allow actions to complete
	tokio::time::sleep(std::time::Duration::from_secs(10)).await;

	// Shutdown the sequence
	Sequence.Shutdown().await;

	println!("Sequence completed");

	Ok(())
}

use serde_json::{json, Value};
use tokio::{
	fs::{File, OpenOptions},
	io::{AsyncReadExt, AsyncWriteExt},
};

use Echo::{
	Enum::Sequence::Action::Error::Enum as Error,
	Struct::Sequence::{
		Action::{Signature::Struct as Signature, Struct as Action},
		Arc,
		Life::Struct as Life,
	},
	Trait::Sequence::Worker::Trait as Worker,
};
