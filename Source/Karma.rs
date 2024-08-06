type StartEnd = Arc<dyn Fn() -> Result<(), ActionError> + Send + Sync>;

