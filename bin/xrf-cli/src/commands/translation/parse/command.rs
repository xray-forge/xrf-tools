use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_report::Status;
use xrf_translation::{ProjectParseOptions, ProjectParseResult, TranslationLanguage, parse_translations};
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::progress::new_logging_job;

#[derive(Default)]
pub struct ParseCommand;

impl GenericCommand for ParseCommand {
  fn operation(&self) -> &'static str {
    "parse"
  }

  /// Create translation parsing command.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to parse xml translations into json sources")
      .arg(
        Arg::new("path")
          .help("Root holding raw xml translations. Repeat to layer roots, highest priority first")
          .short('p')
          .long("path")
          .required(true)
          // Both spellings layer: repeat the flag, or list several values after one of them.
          .action(ArgAction::Append)
          .num_args(1..)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("source")
          .help(
            "How to read the path: auto treats it as an installation only when it declares one, directory ignores any declaration, volumes mounts every archive volume beneath it, installation requires one, containing-installation searches parent directories for one",
          )
          .long("source")
          .default_value("containing-installation")
          .value_parser(["auto", "directory", "volumes", "installation", "containing-installation"]),
      )
      .arg(
        Arg::new("prefix")
          .help("Limit to one logical subtree, such as configs\\text\\eng")
          .long("prefix")
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("language")
          .help("Language every entry read by this run is filed under. Raw xml carries no language, so it is declared rather than guessed")
          .short('l')
          .long("language")
          .required(true)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("output")
          .help("Directory the json sources are written to, merging with any already there")
          .short('o')
          .long("output")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("file")
          .help("Restrict the run to one string table, by file name")
          .long("file")
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("overwrite")
          .help("Replace existing text that differs, instead of keeping what is already there")
          .long("overwrite")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("dry-run")
          .help("Report what would be written without writing it")
          .long("dry-run")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("strict")
          .help("Answer with a check failure when anything was unreadable or off schema")
          .long("strict")
          .action(ArgAction::SetTrue),
      )
  }

  /// Import one language's raw string tables into json sources.
  ///
  /// Reporting is the default and answers success, matching `dialog info`: a file this could not read
  /// costs that file, and an import over somebody else's mod is expected to meet a few. `--strict` is
  /// the mode that judges, and it is the one a build step uses.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let paths: Vec<&PathBuf> = matches
      .get_many::<PathBuf>("path")
      .expect("Expected at least one path to be provided")
      .collect();

    let output: OutputOptions = context.get_output().clone();

    let source: XrayMountMode = XrayMountMode::try_from(
      matches
        .get_one::<String>("source")
        .expect("Expected source mode to default")
        .as_str(),
    )?;

    // One vocabulary for naming roots, so repeating `--path` layers a tree in front of an
    // installation exactly as the desktop app does it.
    let roots: XrayRoots = XrayRoots::new(paths.iter().map(|path| XrayRoot::new(path.to_path_buf(), source)));

    let options: ProjectParseOptions = ProjectParseOptions {
      job: new_logging_job(),
      output: output.clone(),
      roots,
      prefix: matches.get_one::<String>("prefix").cloned(),
      // Rejects `all`: a run files every entry it reads under one key, and there is no such key.
      language: TranslationLanguage::from_str_single(
        matches
          .get_one::<String>("language")
          .expect("Expected valid language for translation"),
      )?,
      output_dir: matches
        .get_one::<PathBuf>("output")
        .expect("Expected valid output folder path to be provided")
        .clone(),
      file: matches.get_one::<String>("file").cloned(),
      is_overwrite: matches.get_flag("overwrite"),
      is_dry_run: matches.get_flag("dry-run"),
    };

    let is_strict: bool = matches.get_flag("strict");
    let result: ProjectParseResult = parse_translations(&options)?;

    Self::print_census(&output, &result);

    // Deposited before the verdict becomes an outcome, so a failing check still reports the findings.
    context.set_result(|| &result)?;

    if is_strict && result.get_status() == Status::Failed {
      return Err(CommandError::new_check_failed(result.get_findings().len().max(1)));
    }

    Ok(())
  }
}

impl ParseCommand {
  fn print_census(output: &OutputOptions, result: &ProjectParseResult) {
    let census = &result.census;

    xrf_output::info!(
      output,
      "Read {} file(s), {} entries as '{}' in {}",
      census.files_read,
      census.entries_read,
      result.language,
      xrf_utils::format_duration(result.duration)
    );

    xrf_output::info!(
      output,
      "{} {} created, {} updated, {} unchanged, {} skipped",
      if result.is_dry_run { "Would have:" } else { "Sources:" },
      census.files_created,
      census.files_updated,
      census.files_unchanged,
      census.files_skipped
    );

    xrf_output::info!(
      output,
      "Entries: {} inserted, {} filled, {} unchanged, {} placeholder(s) added",
      census.entries_inserted,
      census.entries_filled,
      census.entries_unchanged,
      census.placeholders_added
    );

    // Named rather than counted only, because the whole point of keeping existing text is that a
    // caller can see what diverged and decide whether `--overwrite` was what they meant.
    if census.entries_conflicted > 0 {
      xrf_output::info!(
        output,
        "{} entr{} {} existing text",
        census.entries_conflicted,
        if census.entries_conflicted == 1 { "y" } else { "ies" },
        if result.is_dry_run {
          "would have differed from"
        } else {
          "differed from"
        }
      );
    }

    for finding in result.get_findings() {
      xrf_output::verbose!(
        output,
        "{}: {}",
        finding.subject().unwrap_or("<unknown>"),
        finding.message()
      );
    }
  }
}
