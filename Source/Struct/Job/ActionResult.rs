/// Represents the result of an action that has been processed.
///
/// # Fields
///
/// * `Action` - The action that was processed.
/// * `Result` - The result of the action, which is a `Result` type containing either a success message (`String`) or an error message (`String`).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Struct {
	pub Action: Action,
	pub Result: Result<String, String>,
}
