use std::collections::BTreeSet;

use xrf_error::{XrfError, XrfResult};

/// Inlines `#import "module"` lines, each module once, and keeps or drops `#if NAME` / `#else` / `#endif` blocks by
/// whether `NAME` is among `defines`. Directives sit alone on their line; WGSL's `enable` lines are gathered to the top,
/// each once, since they must come before any declaration; everything else passes through unchanged.
///
/// # Errors
///
/// Returns an error for a module `read` cannot give, a directive it does not know, or an unbalanced `#if`.
pub fn compose_shader<'a>(
  entry: &str,
  defines: &[&str],
  read: &dyn Fn(&str) -> XrfResult<&'a str>,
) -> XrfResult<String> {
  let mut output: String = String::new();
  let mut imported: BTreeSet<String> = BTreeSet::new();
  let mut enables: BTreeSet<String> = BTreeSet::new();

  append_module(entry, defines, read, &mut imported, &mut enables, &mut output)?;

  Ok(
    enables
      .into_iter()
      .map(|line| {
        line
          + "
"
      })
      .collect::<String>()
      + &output,
  )
}

fn append_module<'a>(
  name: &str,
  defines: &[&str],
  read: &dyn Fn(&str) -> XrfResult<&'a str>,
  imported: &mut BTreeSet<String>,
  enables: &mut BTreeSet<String>,
  output: &mut String,
) -> XrfResult {
  if !imported.insert(name.to_string()) {
    return Ok(());
  }

  let source: &str = read(name)?;
  // Whether each open `#if` keeps its lines, innermost last.
  let mut kept: Vec<bool> = Vec::new();

  for (index, line) in source.lines().enumerate() {
    let directive: &str = line.trim();
    let is_active: bool = kept.iter().all(|it| *it);
    let at = || format!("'{name}' line {}", index + 1);

    if let Some(rest) = directive.strip_prefix("#if ") {
      kept.push(defines.contains(&rest.trim()));
    } else if directive == "#else" {
      let last: &mut bool = kept
        .last_mut()
        .ok_or_else(|| XrfError::new_invalid_error(format!("'#else' without '#if' at {}", at())))?;

      *last = !*last;
    } else if directive == "#endif" {
      kept
        .pop()
        .ok_or_else(|| XrfError::new_invalid_error(format!("'#endif' without '#if' at {}", at())))?;
    } else if let Some(rest) = directive.strip_prefix("#import ") {
      if is_active {
        let module: &str = rest.trim().trim_matches('"');

        append_module(module, defines, read, imported, enables, output)?;
      }
    } else if directive.starts_with('#') {
      return Err(XrfError::new_invalid_error(format!(
        "Unknown directive '{directive}' at {}",
        at()
      )));
    } else if is_active && directive.starts_with("enable ") {
      enables.insert(directive.to_string());
    } else if is_active {
      output.push_str(line);
      output.push('\n');
    }
  }

  if !kept.is_empty() {
    return Err(XrfError::new_invalid_error(format!("Unclosed '#if' in '{name}'")));
  }

  Ok(())
}
