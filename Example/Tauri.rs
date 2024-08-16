#![allow(non_snake_case)]

struct SimpleWorker;

#[async_trait::async_trait]
impl Worker for SimpleWorker {
	async fn Receive(
		&self,
		Action: Box<dyn Sequence::Action::Trait>,
		Context: &Life,
	) -> Result<(), Error> {
		Action.Execute(Context).await
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let Plan = Arc::new(
		Echo::Struct::Sequence::Plan::Struct::New()
			.WithSignature(Action::Signature::Struct { Name: "Read".to_string() })
			.WithSignature(Action::Signature::Struct { Name: "Write".to_string() })
			.WithFunction("Read", Common::Read::Fn)?
			.WithFunction("Write", Common::Write::Fn)?
			.Build(),
	);

	let Production = Arc::new(Echo::Struct::Sequence::Production::Struct::New());

	let Life = Life {
		Span: Arc::new(dashmap::DashMap::new()),
		Fate: Arc::new(config::Config::default()),
		Cache: Arc::new(tokio::sync::Mutex::new(dashmap::DashMap::new())),
		Karma: Arc::new(dashmap::DashMap::new()),
	};

	let Worker = Arc::new(SimpleWorker);
	let Sequence = Arc::new(Sequence::Struct::New(Worker, Production.clone(), Life));

	// Channel for sending action results
	let (Approval, mut Receipt) = mpsc::unbounded_channel();

	// Spawn worker tasks
	let mut Force = JoinSet::new();

	for _ in 0..4 {
		let Sequence = Sequence.clone();
		let tx = Approval.clone();

		Force.spawn(async move {
			while !Sequence.Time.Get().await {
				if let Some(action) = Sequence.Work.Do().await {
					let result = Sequence.Site.Receive(action, &Sequence.Life).await;
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
				while let Some(result) = Receipt.recv().await {
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
	while let Some(result) = Force.join_next().await {
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
	Enum::Sequence::Action::Error::Enum as Error,
	Struct::Sequence::{self, Action, Life::Struct as Life, Plan},
	Trait::Sequence::Worker,
};

pub mod Common;
