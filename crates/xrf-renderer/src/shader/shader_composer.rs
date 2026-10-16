use std::collections::BTreeSet;

use xrf_error::{XrfError, XrfResult};
use xrf_renderer_core::ShaderLines;

/// Composes a WGSL module with its imports inlined.
pub struct ShaderComposer;

impl ShaderComposer {
  /// Inlines `#import "module"` lines, each module once. Directives sit alone on their line; WGSL's `enable` lines are
  /// gathered to the top, each once, since they must come before any declaration; everything else passes through
  /// unchanged, each run of a module's lines after a marker naming where it came from (`ShaderLines`).
  ///
  /// # Errors
  ///
  /// Returns an error for a module `read` cannot give, or a directive it does not know.
  pub fn compose<'a>(entry: &str, read: &dyn Fn(&str) -> XrfResult<&'a str>) -> XrfResult<String> {
    let mut output: String = String::new();
    let mut imported: BTreeSet<String> = BTreeSet::new();
    let mut enables: BTreeSet<String> = BTreeSet::new();

    Self::append_module(entry, read, &mut imported, &mut enables, &mut output)?;

    Ok(enables.into_iter().map(|line| line + "\n").collect::<String>() + &output)
  }

  /// The modules `entry` composes from, itself included, each once.
  ///
  /// # Errors
  ///
  /// Returns an error for a module `read` cannot give, or a directive it does not know.
  pub fn list_modules<'a>(entry: &str, read: &dyn Fn(&str) -> XrfResult<&'a str>) -> XrfResult<BTreeSet<String>> {
    let mut imported: BTreeSet<String> = BTreeSet::new();
    let mut enables: BTreeSet<String> = BTreeSet::new();
    let mut output: String = String::new();

    Self::append_module(entry, read, &mut imported, &mut enables, &mut output)?;

    Ok(imported)
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

    // A run of the module's lines starts again after each line it does not pass through.
    let mut is_run_started: bool = false;

    for (index, line) in read(name)?.lines().enumerate() {
      let directive: &str = line.trim();

      if let Some(rest) = directive.strip_prefix("#import ") {
        is_run_started = false;
        Self::append_module(rest.trim().trim_matches('"'), read, imported, enables, output)?;
      } else if directive.starts_with('#') {
        return Err(XrfError::new_invalid_error(format!(
          "Unknown directive '{directive}' at '{name}' line {}",
          index + 1
        )));
      } else if directive.starts_with("enable ") {
        is_run_started = false;
        enables.insert(directive.to_string());
      } else {
        if !is_run_started {
          is_run_started = true;
          output.push_str(&ShaderLines::mark(name, index + 1));
          output.push('\n');
        }

        output.push_str(line);
        output.push('\n');
      }
    }

    Ok(())
  }
}
