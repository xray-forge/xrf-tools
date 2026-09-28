use full_moon::ast::{Expression, Var};
use full_moon::tokenizer::{StringLiteralQuoteType, TokenType};

use crate::lua_scope::LuaScope;
use crate::xray_lua_binding::XRayLuaBinding;

/// One argument of a Lua call, as far as a reader that never runs the script can know it.
#[derive(Clone, Debug, PartialEq)]
pub enum XRayLuaValue {
  String(String),
  Number(f64),
  Boolean(bool),
  /// A name no local or parameter binds where it is written, or a dotted name - `blend.srcalpha`, `t_rt` - kept
  /// verbatim, since nothing here evaluates it.
  Name(String),
  /// A bare name a local or a parameter binds where it is written, and what binds it.
  Local {
    name: String,
    binding: XRayLuaBinding,
  },
  /// Anything a reader cannot reduce to the above: a table, an operation, a call, a string it cannot decode.
  Other,
}

impl XRayLuaValue {
  /// Reads one argument expression as far as it can be read without running the script, a bare name through what
  /// `scope` binds it to.
  pub(crate) fn of(expression: &Expression, scope: &LuaScope) -> Self {
    match expression {
      Expression::String(token) => match token.token().token_type() {
        TokenType::StringLiteral {
          literal, quote_type, ..
        } => Self::read_string(literal, *quote_type).map_or(Self::Other, Self::String),
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
      Expression::Var(Var::Name(token)) => {
        let name: String = token.token().to_string();

        match scope.resolve(&name) {
          Some(binding) => Self::Local {
            binding: binding.clone(),
            name,
          },
          None => Self::Name(name),
        }
      }
      // `blend.srcalpha` is a variable expression, and its own spelling is the whole of what it says here: the
      // renderer's tables are what give it a value, and this crate reads scripts rather than running them.
      Expression::Var(variable) => Self::Name(variable.to_string().trim().to_owned()),
      _ => Self::Other,
    }
  }

  /// What a local bound to this value holds.
  pub(crate) fn to_binding(&self) -> XRayLuaBinding {
    match self {
      Self::String(value)
      | Self::Local {
        binding: XRayLuaBinding::String(value),
        ..
      } => XRayLuaBinding::String(value.clone()),
      _ => XRayLuaBinding::Other,
    }
  }

  /// A string literal's value from its source text: a long bracket's as written, and a quoted one's with its escapes
  /// decoded as LuaJIT's lexer decodes them (`lj_lex.c`); `None` for one it would refuse or that is not UTF-8.
  fn read_string(literal: &str, quote_type: StringLiteralQuoteType) -> Option<String> {
    match quote_type {
      // A long bracket processes no escapes and drops a newline opening it.
      StringLiteralQuoteType::Brackets => Some(Self::skip_newline(literal).to_owned()),
      _ => Self::unescape(literal),
    }
  }

  /// The text after one newline sequence opening it, `\n`, `\r`, `\r\n` or `\n\r`, as the lexer's `inclinenumber`.
  fn skip_newline(text: &str) -> &str {
    let mut characters = text.chars();

    match (characters.next(), characters.next()) {
      (Some('\n'), Some('\r')) | (Some('\r'), Some('\n')) => &text[2..],
      (Some('\n' | '\r'), _) => &text[1..],
      _ => text,
    }
  }

  /// A quoted string's value, every escape `lj_lex.c` knows decoded, or `None` for one it does not.
  fn unescape(literal: &str) -> Option<String> {
    let bytes: &[u8] = literal.as_bytes();
    let mut value: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut at: usize = 0;

    while at < bytes.len() {
      if bytes[at] != b'\\' {
        value.push(bytes[at]);
        at += 1;

        continue;
      }

      let escape: u8 = *bytes.get(at + 1)?;

      at += 2;

      match escape {
        b'a' => value.push(0x07),
        b'b' => value.push(0x08),
        b'f' => value.push(0x0C),
        b'n' => value.push(b'\n'),
        b'r' => value.push(b'\r'),
        b't' => value.push(b'\t'),
        b'v' => value.push(0x0B),
        b'\\' | b'"' | b'\'' => value.push(escape),
        // An escaped line break is a line break, a two-byte one counted as one.
        b'\n' | b'\r' => {
          value.push(b'\n');

          if bytes
            .get(at)
            .is_some_and(|next| matches!(next, b'\n' | b'\r') && *next != escape)
          {
            at += 1;
          }
        }
        // Exactly two hexadecimal digits.
        b'x' => {
          value.push(Self::read_hex(bytes.get(at..at + 2)?)? as u8);
          at += 2;
        }
        // Skips the whitespace after it, line breaks included.
        b'z' => {
          while bytes.get(at).is_some_and(|byte| Self::is_space(*byte)) {
            at += 1;
          }
        }
        // `\u{XXX}`, a code point written out as UTF-8.
        b'u' => {
          if bytes.get(at) != Some(&b'{') {
            return None;
          }

          let close: usize = at + bytes[at..].iter().position(|byte| *byte == b'}')?;
          let character: char = char::from_u32(Self::read_hex(&bytes[at + 1..close])?)?;

          value.extend_from_slice(character.encode_utf8(&mut [0; 4]).as_bytes());
          at = close + 1;
        }
        // Up to three decimal digits, no more than a byte.
        b'0'..=b'9' => {
          let mut code: u32 = u32::from(escape - b'0');

          for _ in 0..2 {
            match bytes.get(at) {
              Some(digit) if digit.is_ascii_digit() => {
                code = code * 10 + u32::from(digit - b'0');
                at += 1;
              }
              _ => break,
            }
          }

          value.push(u8::try_from(code).ok()?);
        }
        _ => return None,
      }
    }

    String::from_utf8(value).ok()
  }

  /// The value of a run of hexadecimal digits, `None` for an empty run, any other character, or one past 32 bits.
  fn read_hex(digits: &[u8]) -> Option<u32> {
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_hexdigit) {
      return None;
    }

    u32::from_str_radix(std::str::from_utf8(digits).ok()?, 16).ok()
  }

  /// `lj_char_isspace`: a space, a tab, a line break, a vertical tab or a form feed.
  const fn is_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r')
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
      Self::Name(value) | Self::Local { name: value, .. } => Some(value),
      _ => None,
    }
  }
}
