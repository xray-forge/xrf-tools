use xrf_error::{XrfError, XrfResult};

/// One `fsgame.ltx` line: an alias and where it points.
///
/// Despite the extension, the file has no LTX sections. Each value has three required and three optional fields:
/// `recurs | notif | root | add | ext | description`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FsgameDeclaration {
  /// Alias including its dollar signs, such as `$game_data$`.
  pub alias: String,
  /// Whether the engine scans the path recursively.
  pub is_recursive: bool,
  /// Whether change notifications are enabled for the path. This crate stores the flag but does not act on it.
  pub is_notified: bool,
  /// Alias that supplies the base path for this declaration.
  pub root_alias: String,
  /// Directory appended to the root, absent when the alias names the root itself.
  pub addition: Option<String>,
  /// Optional raw file-filter field, such as `*.ogf;*.omf`.
  pub extensions: Option<String>,
  /// Optional human-readable description from the final tuple field.
  pub description: Option<String>,
}

impl FsgameDeclaration {
  /// Parses one declaration line.
  ///
  /// Surrounding whitespace around the alias and fields is ignored.
  ///
  /// # Errors
  ///
  /// Returns an error when the line has no `=` or contains fewer than the three required fields.
  pub fn parse(line: &str) -> XrfResult<Self> {
    let Some((alias, value)) = line.split_once('=') else {
      return Err(XrfError::new_invalid_error(format!(
        "fsgame declaration has no '=': {line}"
      )));
    };

    let alias: &str = alias.trim();
    let fields: Vec<&str> = value.split('|').map(str::trim).collect();

    if fields.len() < 3 {
      return Err(XrfError::new_invalid_error(format!(
        "fsgame declaration '{alias}' has {} field(s), expected at least 3",
        fields.len()
      )));
    }

    Ok(Self {
      addition: Self::optional(fields.get(3)),
      alias: alias.to_string(),
      description: Self::optional(fields.get(5)),
      extensions: Self::optional(fields.get(4)),
      is_notified: Self::flag(fields[1]),
      is_recursive: Self::flag(fields[0]),
      root_alias: fields[2].to_string(),
    })
  }

  /// Checks whether a line is a non-comment declaration candidate.
  pub fn is_declaration(line: &str) -> bool {
    let line: &str = line.trim();

    !line.is_empty() && !line.starts_with(';') && line.contains('=')
  }

  /// Returns the addition without trailing separators, or `None` when it is empty.
  pub fn get_addition_segment(&self) -> Option<&str> {
    self
      .addition
      .as_deref()
      .map(|addition| addition.trim_end_matches(['\\', '/']))
      .filter(|addition| !addition.is_empty())
  }

  /// Treats only `true`, case-insensitively, as an enabled flag.
  fn flag(field: &str) -> bool {
    field.eq_ignore_ascii_case("true")
  }

  fn optional(field: Option<&&str>) -> Option<String> {
    field
      .map(|field| field.trim())
      .filter(|field| !field.is_empty())
      .map(ToString::to_string)
  }
}
