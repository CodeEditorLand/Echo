#[derive(Debug, Error)]
pub enum Enum {
	#[error("Invalid License: {0}")]
	InvalidLicense(String),
	#[error("Execution Error: {0}")]
	ExecutionError(String),
}

use thiserror::Error;
