/// A suggestion the console's drawer lists under its field as a line is typed: the text taking it puts in the field,
/// and a hint shown beside it, dimmer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeveloperSuggestion {
  pub text: String,
  pub hint: String,
}
