#[derive(Error, Debug)]
pub enum Enum {
	#[error("Invalid license: {0}")]
	InvalidLicense(String),
	#[error("Execution error: {0}")]
	ExecutionError(String),
	#[error("Routing error: {0}")]
	RoutingError(String),
	#[error("Cancellation error: {0}")]
	CancellationError(String),
}

use thiserror::Error;
