/// Where a composed shader's lines came from: each run of one module's lines starts with a marker naming the module and
/// the run's first line in it, which the composed lines an error names are told back by.
pub struct ShaderLines;

impl ShaderLines {
  /// What starts a run's marker, before the module's name, a colon and the run's first line.
  pub const MARKER: &'static str = "// module: ";
  /// What a compiler's error names a composed line by, before the line and its column.
  const ERROR_PLACE: &'static str = "wgsl:";

  /// The marker starting a run of `module`'s lines at its `line`, counted from one.
  pub fn mark(module: &str, line: usize) -> String {
    format!("{}{module}:{line}", Self::MARKER)
  }

  /// The module and its line, counted from one, that the composed `line` came from; none where no marker precedes it.
  pub fn locate(code: &str, line: usize) -> Option<(&str, usize)> {
    let lines: Vec<&str> = code.lines().take(line.saturating_sub(1)).collect();

    lines.iter().enumerate().rev().find_map(|(index, text)| {
      let (module, first) = text.strip_prefix(Self::MARKER)?.rsplit_once(':')?;

      Some((module, first.parse::<usize>().ok()? + (line - 2 - index)))
    })
  }

  /// `error` with each composed place it names, `wgsl:LINE:COLUMN`, followed by the module and line it came from.
  pub fn annotate(code: &str, error: &str) -> String {
    let mut annotated: String = String::with_capacity(error.len());
    let mut rest: &str = error;

    while let Some(at) = rest.find(Self::ERROR_PLACE) {
      let (before, after) = rest.split_at(at + Self::ERROR_PLACE.len());
      let line_digits: usize = after.bytes().take_while(u8::is_ascii_digit).count();
      let column_digits: usize = after[line_digits..]
        .strip_prefix(':')
        .map_or(0, |column| column.bytes().take_while(u8::is_ascii_digit).count() + 1);
      let end: usize = line_digits + column_digits;

      annotated.push_str(before);
      annotated.push_str(&after[..end]);

      if let Some((module, module_line)) = after[..line_digits]
        .parse::<usize>()
        .ok()
        .and_then(|line| Self::locate(code, line))
      {
        annotated.push_str(&format!(" ({module}:{module_line})"));
      }

      rest = &after[end..];
    }

    annotated.push_str(rest);
    annotated
  }
}
