use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_report::Status;
use xrf_translation::{TranslationLanguage, TranslationParseOptions, TranslationParseResult, TranslationParser};

use crate::core::arguments::{LinkPolicyArgument, XrayRootsArguments};
use crate::core::command::{CommandContext, CommandError, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

#[derive(Default)]
pub struct ParseCommand;

impl GenericCommand for ParseCommand {
  fn operation(&self) -> &'static str {
    "parse"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = XrayRootsArguments::declare(
      command.about("Parse XML translations into JSON sources"),
      "Root holding raw xml translations. Repeat to layer roots, highest priority first",
      "Limit to one logical subtree, such as configs\\text\\eng",
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
        Arg::new("dest")
          .help("Directory the json sources are written to, merging with any already there")
          .short('d')
          .long("dest")
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
      );

    LinkPolicyArgument::declare(command)
  }

  /// Import one language's raw string tables into json sources.
  ///
  /// Reporting is the default and answers success, matching `dialog info`: a file this could not read
  /// costs that file, and an import over somebody else's mod is expected to meet a few. `--strict` is
  /// the mode that judges, and it is the one a build step uses.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let options: TranslationParseOptions = TranslationParseOptions {
      job: CliJob::new_logging(),
      output: output.clone(),
      roots: XrayRootsArguments::of(matches)?,
      prefix: XrayRootsArguments::prefix_of(matches).map(str::to_owned),
      // Rejects `all`: a run files every entry it reads under one key, and there is no such key.
      language: TranslationLanguage::from_str_single(
        matches
          .get_one::<String>("language")
          .expect("Expected valid language for translation"),
      )?,
      output_dir: matches
        .get_one::<PathBuf>("dest")
        .expect("Expected valid output folder path to be provided")
        .clone(),
      file: matches.get_one::<String>("file").cloned(),
      is_overwrite: matches.get_flag("overwrite"),
      is_dry_run: matches.get_flag("dry-run"),
      link_policy: LinkPolicyArgument::of(matches),
    };

    let is_strict: bool = matches.get_flag("strict");
    let result: TranslationParseResult = TranslationParser::parse(&options)?;

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
  fn print_census(output: &OutputOptions, result: &TranslationParseResult) {
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
