use std::collections::BTreeSet;

use xrf_error::{XrfError, XrfResult};

/// Inlines `#import "module"` lines, each module once. Directives sit alone on their line; WGSL's `enable` lines are
/// gathered to the top, each once, since they must come before any declaration; everything else passes through
/// unchanged.
///
/// # Errors
///
/// Returns an error for a module `read` cannot give, or a directive it does not know.
pub fn compose_shader<'a>(entry: &str, read: &dyn Fn(&str) -> XrfResult<&'a str>) -> XrfResult<String> {
  let mut output: String = String::new();
  let mut imported: BTreeSet<String> = BTreeSet::new();
  let mut enables: BTreeSet<String> = BTreeSet::new();

  append_module(entry, read, &mut imported, &mut enables, &mut output)?;

  Ok(enables.into_iter().map(|line| line + "\n").collect::<String>() + &output)
}

fn append_module<'a>(
  name: &str,
  read: &dyn Fn(&str) -> XrfResult<&'a str>,
  imported: &mut BTreeSet<String>,
  enables: &mut BTreeSet<String>,
  output: &mut String,
) -> XrfResult {
  if !imported.insert(name.to_string()) {
    return Ok(());
  }

  for (index, line) in read(name)?.lines().enumerate() {
    let directive: &str = line.trim();

    if let Some(rest) = directive.strip_prefix("#import ") {
      append_module(rest.trim().trim_matches('"'), read, imported, enables, output)?;
    } else if directive.starts_with('#') {
      return Err(XrfError::new_invalid_error(format!(
        "Unknown directive '{directive}' at '{name}' line {}",
        index + 1
      )));
    } else if directive.starts_with("enable ") {
      enables.insert(directive.to_string());
    } else {
      output.push_str(line);
      output.push('\n');
    }
  }

  Ok(())
}
