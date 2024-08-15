#[derive(Debug, Error)]
pub enum Enum {
	#[error("Invalid License: {0}")]
	InvalidLicense(String),
	#[error("Execution Error: {0}")]
	ExecutionError(String),
	#[error("Routing error: {0}")]
	RoutingError(String),
	#[error("Cancellation error: {0}")]
	CancellationError(String),
}

use thiserror::Error;
