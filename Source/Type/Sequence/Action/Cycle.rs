pub type Type = Arc<dyn Fn() -> Result<(), Error> + Send + Sync>;
