#[derive(Debug, Error)]
pub enum Enum {
	#[error("Invalid License: {0}")]
	License(String),
	#[error("Execution Error: {0}")]
	Execution(String),
	#[error("Routing error: {0}")]
	Routing(String),
	#[error("Cancellation error: {0}")]
	Cancellation(String),
}

use thiserror::Error;
