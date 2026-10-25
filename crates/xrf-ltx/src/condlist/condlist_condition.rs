use super::source_span::SourceSpan;

/// A condition or effect in a condition-list branch.
#[derive(Clone, Debug, PartialEq)]
pub enum CondlistCondition {
  InfoPortion {
    name: String,
    required: bool,
    span: SourceSpan,
  },
  Probability {
    value: f64,
    span: SourceSpan,
  },
  Function {
    expected: bool,
    name: String,
    parameters: Option<Vec<String>>,
    span: SourceSpan,
  },
}
