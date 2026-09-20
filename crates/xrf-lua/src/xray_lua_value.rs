use full_moon::ast::Expression;
use full_moon::tokenizer::TokenType;

/// One argument of a Lua call, as far as a reader that never runs the script can know it.
#[derive(Clone, Debug, PartialEq)]
pub enum XRayLuaValue {
  String(String),
  Number(f64),
  Boolean(bool),
  /// A name or a dotted name - `blend.srcalpha`, `t_base` - kept verbatim, since nothing here evaluates it.
  Name(String),
  /// Anything a reader cannot reduce to the above: a table, an operation, a call.
  Other,
}

impl XRayLuaValue {
  /// Reads one argument expression as far as it can be read without running the script.
  pub(crate) fn of(expression: &Expression) -> Self {
    match expression {
      Expression::String(token) => match token.token().token_type() {
        TokenType::StringLiteral { literal, .. } => Self::String(literal.to_string()),
        _ => Self::Other,
      },
      Expression::Number(token) => token
        .token()
        .to_string()
        .parse::<f64>()
        .map_or(Self::Other, Self::Number),
      // `true` and `false` arrive as symbols, as does `nil`, which is neither of them.
      Expression::Symbol(token) => match token.token().to_string().as_str() {
        "true" => Self::Boolean(true),
        "false" => Self::Boolean(false),
        other => Self::Name(other.to_owned()),
      },
      // `blend.srcalpha` is a variable expression, and its own spelling is the whole of what it says here: the
      // renderer's tables are what give it a value, and this crate reads scripts rather than running them.
      Expression::Var(variable) => Self::Name(variable.to_string().trim().to_owned()),
      _ => Self::Other,
    }
  }

  /// The string this argument is, for one that is a literal string.
  pub fn as_string(&self) -> Option<&str> {
    match self {
      Self::String(value) => Some(value),
      _ => None,
    }
  }

  /// The number this argument is, for one that is a literal number.
  pub fn as_number(&self) -> Option<f64> {
    match self {
      Self::Number(value) => Some(*value),
      _ => None,
    }
  }

  /// Whether this argument is the literal `true`, which is how every script state switch is written.
  pub fn is_true(&self) -> bool {
    matches!(self, Self::Boolean(true))
  }

  /// The name this argument is, for one that names something rather than stating it.
  pub fn as_name(&self) -> Option<&str> {
    match self {
      Self::Name(value) => Some(value),
      _ => None,
    }
  }
}
