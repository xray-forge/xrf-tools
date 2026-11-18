use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_utils::{StagedFile, format_path};

use super::group_reference::GroupReference;
use super::markdown_renderer::{GENERATED_HEADER, ReferenceMarkdownRenderer};
use super::report::DocsGenerateReport;
use crate::core::command::{CommandContext, CommandError, CommandGroup, CommandResult, GenericCommand};

#[derive(Default)]
pub(crate) struct GenerateCommand;

impl GenericCommand for GenerateCommand {
  fn operation(&self) -> &'static str {
    "generate"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Generate the markdown reference of every CLI command")
      .arg(
        Arg::new("dest")
          .help("Directory the reference is generated into, whole")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("check")
          .help("Verify existing documentation is up to date instead of writing it")
          .short('c')
          .long("check")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("force")
          .help(
            "Write where the directory holds Markdown pages this command did not generate, replacing those of the same \
             name and leaving the others; without it, one there is refused before any is written",
          )
          .short('f')
          .long("force")
          .conflicts_with("check")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output_dir: &PathBuf = matches
      .get_one::<PathBuf>("dest")
      .expect("Expected valid output directory to be provided");

    let output: OutputOptions = context.get_output().clone();

    let groups: Vec<GroupReference> = CommandGroup::list_all()
      .iter()
      .map(GroupReference::from_group)
      .collect();

    let pages: Vec<(String, String)> = ReferenceMarkdownRenderer::render_pages(&groups);

    if matches.get_flag("check") {
      Self::check_pages(context, output_dir, &pages, &output)
    } else {
      Self::write_pages(context, output_dir, &pages, matches.get_flag("force"), &output)
    }
  }
}

impl GenerateCommand {
  /// Replaces generated Markdown pages, removing those left over after a group rename.
  ///
  /// A page is this command's when it opens with the generated header. Any other Markdown page in the directory is
  /// someone else's: unforced, one refuses the run before anything is written; forced, one of a generated page's name is
  /// replaced and the rest are left where they are.
  fn write_pages(
    context: &mut CommandContext,
    directory: &Path,
    pages: &[(String, String)],
    is_forced: bool,
    output: &OutputOptions,
  ) -> CommandResult {
    fs::create_dir_all(directory)?;

    let (removed, foreign): (Vec<String>, Vec<String>) = Self::sort_existing_pages(directory, pages)?;

    Self::assert_no_foreign_pages(directory, &foreign, is_forced)?;

    for name in &foreign {
      if !Self::is_page(pages, name) {
        xrf_output::info!(output, "Leaving a page it did not generate: {name}");
      }
    }

    for name in &removed {
      xrf_output::info!(output, "Removing stale documentation page: {name}");
      fs::remove_file(directory.join(name))?;
    }

    for (name, content) in pages {
      StagedFile::write(&directory.join(name), content.as_bytes())?;
    }

    xrf_output::success!(
      output,
      "Generated {} documentation pages in {}",
      pages.len(),
      format_path(directory)
    );

    context.set_result(|| DocsGenerateReport::written(directory, pages.len(), removed))
  }

  fn check_pages(
    context: &mut CommandContext,
    directory: &Path,
    pages: &[(String, String)],
    output: &OutputOptions,
  ) -> CommandResult {
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
      xrf_output::success!(
        output,
        "CLI documentation in {} is up to date ({} pages)",
        format_path(directory),
        pages.len()
      );
    } else {
      xrf_output::failure!(
        output,
        "CLI documentation in {} is stale, regenerate it with 'xrf-cli docs generate':",
        format_path(directory)
      );

      for name in &stale {
        xrf_output::failure!(output, "  {name}");
      }
    }

    let drifted: usize = stale.len();

    // Deposited before the verdict becomes an outcome, so a failing check still reports the drift that explains it.
    context.set_result(|| DocsGenerateReport::checked(directory, pages.len(), stale))?;

    if drifted == 0 {
      Ok(())
    } else {
      Err(CommandError::new_check_failed(drifted))
    }
  }

  /// The directory's Markdown pages that need a decision before a write: stale ones the command generated and no longer
  /// does, then those it did not generate, whatever their name; each sorted by name.
  fn sort_existing_pages(directory: &Path, pages: &[(String, String)]) -> CommandResult<(Vec<String>, Vec<String>)> {
    let mut stale: Vec<String> = Vec::new();
    let mut foreign: Vec<String> = Vec::new();

    for entry in fs::read_dir(directory)? {
      let entry: fs::DirEntry = entry?;
      let Ok(name) = entry.file_name().into_string() else {
        continue;
      };

      if !entry.file_type()?.is_file() || !Self::is_markdown(&name) {
        continue;
      }

      let is_generated: bool =
        fs::read_to_string(entry.path()).is_ok_and(|content| content.starts_with(GENERATED_HEADER));

      if !is_generated {
        foreign.push(name);
      } else if !Self::is_page(pages, &name) {
        stale.push(name);
      }
    }

    stale.sort();
    foreign.sort();

    Ok((stale, foreign))
  }

  /// Refuses, unless forced, a directory holding Markdown pages this command did not generate.
  fn assert_no_foreign_pages(directory: &Path, foreign: &[String], is_forced: bool) -> CommandResult {
    if foreign.is_empty() || is_forced {
      return Ok(());
    }

    Err(
      XrfError::new_io_error(
        format!(
          "{} holds Markdown pages this command did not generate ({}); force it to write beside them, replacing those \
           of a generated page's name",
          format_path(directory),
          foreign.join(", ")
        ),
        ErrorKind::AlreadyExists,
      )
      .into(),
    )
  }

  /// Whether a file name is one of the generated pages, compared without case as the file systems it lands on compare.
  fn is_page(pages: &[(String, String)], name: &str) -> bool {
    pages.iter().any(|(page, _)| page.eq_ignore_ascii_case(name))
  }

  fn is_markdown(name: &str) -> bool {
    Path::new(name)
      .extension()
      .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
  }

  fn list_unexpected_pages(directory: &Path, pages: &[(String, String)]) -> Vec<String> {
    let Ok(entries) = fs::read_dir(directory) else {
      return Vec::new();
    };

    entries
      .flatten()
      .filter_map(|entry| entry.file_name().into_string().ok())
      .filter(|name| Self::is_markdown(name) && !pages.iter().any(|(page, _)| page == name))
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::{Path, PathBuf};

  use clap::ArgMatches;
  use xrf_test_utils::GeneratedResources;

  use super::{GENERATED_HEADER, GenerateCommand};
  use crate::core::command::command_testing::run_command;
  use crate::core::command::{CommandResult, GenericCommand};

  fn parse_matches(extra: &[&str]) -> ArgMatches {
    let mut arguments = vec!["generate", "--dest", "docs"];

    arguments.extend_from_slice(extra);

    GenerateCommand.init().try_get_matches_from(arguments).unwrap()
  }

  fn generate(directory: &Path, extra: &[&str]) -> CommandResult {
    let mut arguments: Vec<String> = vec![
      String::from("generate"),
      String::from("--dest"),
      directory.display().to_string(),
      String::from("--silent"),
    ];

    arguments.extend(extra.iter().map(|argument| String::from(*argument)));

    run_command(&GenerateCommand, &arguments)
  }

  /// A reference directory generated once, plus the pages named, each with its content.
  fn create_reference(name: &str, extra: &[(&str, String)]) -> CommandResult<PathBuf> {
    let directory: PathBuf = GeneratedResources::create_dir(&format!("docs_generate/{name}"))?;

    generate(&directory, &[])?;

    for (page, content) in extra {
      fs::write(directory.join(page), content)?;
    }

    Ok(directory)
  }

  #[test]
  fn check_mode_is_disabled_by_default() {
    assert!(!parse_matches(&[]).get_flag("check"));
    assert!(parse_matches(&["--check"]).get_flag("check"));
  }

  #[test]
  fn removes_a_stale_page_it_generated() -> CommandResult {
    let directory: PathBuf = create_reference(
      "stale",
      &[("renamed.md", format!("{GENERATED_HEADER}\n\n# Renamed commands\n"))],
    )?;

    generate(&directory, &[])?;

    assert!(!directory.join("renamed.md").exists());
    assert!(directory.join("README.md").is_file());

    Ok(())
  }

  #[test]
  fn refuses_unforced_a_page_it_did_not_generate_and_writes_nothing() -> CommandResult {
    let directory: PathBuf = create_reference(
      "foreign",
      &[
        ("notes.md", String::from("# Notes\n")),
        ("renamed.md", format!("{GENERATED_HEADER}\n\n# Renamed commands\n")),
        ("README.md", String::from("# Hand-written index\n")),
      ],
    )?;

    let error: String = generate(&directory, &[])
      .expect_err("a page it did not generate is there")
      .to_string();

    assert!(error.contains("(README.md, notes.md)"), "{error}");
    assert!(directory.join("renamed.md").is_file());
    assert_eq!(
      fs::read_to_string(directory.join("README.md"))?,
      "# Hand-written index\n"
    );

    Ok(())
  }

  #[test]
  fn forced_replaces_a_page_of_its_name_and_leaves_the_others() -> CommandResult {
    let directory: PathBuf = create_reference(
      "forced",
      &[
        ("notes.md", String::from("# Notes\n")),
        ("README.md", String::from("# Hand-written index\n")),
      ],
    )?;

    generate(&directory, &["--force"])?;

    assert_eq!(fs::read_to_string(directory.join("notes.md"))?, "# Notes\n");
    assert!(fs::read_to_string(directory.join("README.md"))?.starts_with(GENERATED_HEADER));

    Ok(())
  }
}
