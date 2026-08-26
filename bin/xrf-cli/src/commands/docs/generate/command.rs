use std::fs;
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;

use super::command_reference::GroupReference;
use super::markdown_renderer::ReferenceMarkdownRenderer;
use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct GenerateCommand;

impl GenericCommand for GenerateCommand {
  fn operation(&self) -> &'static str {
    "generate"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to generate markdown reference for all CLI commands")
      .arg(
        Arg::new("output")
          .help("Path to fully generated documentation directory")
          .short('o')
          .long("output")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("check")
          .help("Verify existing documentation is up to date instead of writing it")
          .short('c')
          .long("check")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output_dir: &PathBuf = matches
      .get_one::<PathBuf>("output")
      .expect("Expected valid output directory to be provided");

    let output: OutputOptions = context.get_output().clone();

    let groups: Vec<GroupReference> = crate::registry::setup_command_groups()
      .iter()
      .map(GroupReference::from_group)
      .collect();

    let pages: Vec<(String, String)> = ReferenceMarkdownRenderer::render_pages(&groups);

    if matches.get_flag("check") {
      Self::check_pages(output_dir, &pages, &output)
    } else {
      Self::write_pages(output_dir, &pages, &output)
    }
  }
}

impl GenerateCommand {
  /// Replaces generated Markdown pages, including pages left over after a group rename.
  fn write_pages(directory: &Path, pages: &[(String, String)], output: &OutputOptions) -> CommandResult {
    fs::create_dir_all(directory)?;

    for name in Self::list_unexpected_pages(directory, pages) {
      xrf_output::info!(output, "Removing stale documentation page: {name}");
      fs::remove_file(directory.join(name))?;
    }

    for (name, content) in pages {
      fs::write(directory.join(name), content)?;
    }

    xrf_output::info!(
      output,
      "Generated {} documentation pages in {}",
      pages.len(),
      directory.display()
    );

    Ok(())
  }

  fn check_pages(directory: &Path, pages: &[(String, String)], output: &OutputOptions) -> CommandResult {
    let mut stale: Vec<String> = Vec::new();

    for (name, content) in pages {
      match fs::read_to_string(directory.join(name)) {
        // Windows checkouts may materialize committed pages with CRLF endings.
        Ok(existing) if existing.replace("\r\n", "\n") == *content => {}
        Ok(_) => stale.push(format!("outdated: {name}")),
        Err(_) => stale.push(format!("missing: {name}")),
      }
    }

    for name in Self::list_unexpected_pages(directory, pages) {
      stale.push(format!("unexpected: {name}"));
    }

    if stale.is_empty() {
      xrf_output::info!(
        output,
        "CLI documentation in {} is up to date ({} pages)",
        directory.display(),
        pages.len()
      );

      Ok(())
    } else {
      xrf_output::failure!(
        output,
        "CLI documentation in {} is stale, regenerate it with 'xrf-cli docs generate':",
        directory.display()
      );

      for name in &stale {
        xrf_output::failure!(output, "  {name}");
      }

      Err(CommandError::new_check_failed(stale.len()))
    }
  }

  fn list_unexpected_pages(directory: &Path, pages: &[(String, String)]) -> Vec<String> {
    let Ok(entries) = fs::read_dir(directory) else {
      return Vec::new();
    };

    entries
      .flatten()
      .filter_map(|entry| entry.file_name().into_string().ok())
      .filter(|name| name.ends_with(".md") && !pages.iter().any(|(page, _)| page == name))
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use clap::ArgMatches;

  use super::GenerateCommand;
  use crate::core::generic_command::GenericCommand;

  fn parse_matches(extra: &[&str]) -> ArgMatches {
    let mut arguments = vec!["generate", "--output", "docs"];

    arguments.extend_from_slice(extra);

    GenerateCommand.init().try_get_matches_from(arguments).unwrap()
  }

  #[test]
  fn check_mode_is_disabled_by_default() {
    assert!(!parse_matches(&[]).get_flag("check"));
    assert!(parse_matches(&["--check"]).get_flag("check"));
  }
}
