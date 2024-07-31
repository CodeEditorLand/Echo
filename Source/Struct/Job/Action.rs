/// Represents different types of actions that can be performed.
///
/// # Variants
///
/// * `Read` - Represents a read action with a specified file path.
/// * `Write` - Represents a write action with a specified file path and content.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Struct {
	pub Target: Option<Box<Struct>>,
	pub Metadata: Metadata,
}

impl Struct {
	/// Executes the action and checks if it is followed by another action.
	pub fn Fn(&self) {
		// Perform the logic for the current action
		println!("Executing action: {}", self.Metadata);

		// Check if there is a subsequent action
		if let Some(Next) = &self.Target {
			println!("This action is followed by another action: {}", Next.Metadata);

			// Perform additional logic if needed
			// Next.Fn(self.Metadata.clone());
			Next.Fn();
		}
	}
}

use serde::{Deserialize, Serialize};
use std::fs::Metadata;

pub type Metadata = serde_json::Value;
