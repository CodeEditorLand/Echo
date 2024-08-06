pub type Type = Arc<dyn Fn() -> Result<(), crate::Enum::Sequence::Action::Error::Enum> + Send + Sync>;
