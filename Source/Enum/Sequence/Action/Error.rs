#[derive(Debug, Error)]
pub enum Enum {
	// TODO: ADD MORE LIKE (InvalidLicense, ExecutionError, RoutingErro CancellationError)
	#[error("Invalid License: {0}")]
	InvalidLicense(String),
	#[error("Execution Error: {0}")]
	ExecutionError(String),
}

use thiserror::Error;
