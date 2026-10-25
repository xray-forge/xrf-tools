use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_export::{ExternFormat, ExternManifestParser, ParsedExternManifest};
use xrf_output::OutputOptions;
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_utils::{LineEndings, format_path};

use super::externs_check_report::ExternsCheckReport;
use super::report::ExternsExportReport;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;

/// Generate or verify a stable extern manifest from TypeScript declarations.
#[derive(Default)]
pub struct ExportCommand;

impl GenericCommand for ExportCommand {
  fn operation(&self) -> &'static str {
    "export"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Export TypeScript extern declarations as JSON, XML, or HTML")
      .arg(
        Arg::new("path")
          .help("Root directory containing TypeScript declaration sources")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Artifact to create or replace, or with --check to verify")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("check")
          .help("Verify the artifact against the declarations without writing")
          .short('c')
          .long("check")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("format")
          .help("Artifact format, inferred from the artifact's extension when omitted")
          .long("format")
          .value_parser(ExternFormat::NAMES),
      )
      .arg(
        Arg::new("line-endings")
          .help("Override generated line endings")
          .long("line-endings")
          .value_parser(["lf", "crlf"]),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let declarations_root: &PathBuf = matches.get_one("path").expect("Expected declarations root");
    let path: &PathBuf = matches.get_one("dest").expect("Expected an artifact path");

    let line_endings: Option<LineEndings> = matches
      .get_one::<String>("line-endings")
      .map(|value: &String| LineEndings::from_str(value))
      .transpose()?;

    let output: OutputOptions = context.get_output().clone();
    let format: ExternFormat = Self::resolve_format(matches, path)?;

    if matches.get_flag("check") {
      return Self::check_artifact(context, declarations_root, path, format, line_endings);
    }

    // Writing judges nothing, so invalid declarations are simply work the command cannot do.
    let parsed: ParsedExternManifest = ExternManifestParser::new().parse_directory(declarations_root)?;
    let externs: usize = parsed.manifest.exports.len();

    parsed.manifest.write(path, format, line_endings)?;

    xrf_output::success!(output, "Exported {externs} externs to '{}'.", format_path(path));

    context.set_result(|| ExternsExportReport::new(declarations_root, path, format, externs))
  }
}

impl ExportCommand {
  /// The one check `--check` runs: the artifact still describes the declarations.
  const CHECK: &'static str = "externs.artifact";
  /// Declarations that did not parse, so nothing could be compared.
  const RULE_DECLARATIONS: &'static str = "externs.declarations";
  /// An artifact that does not parse as its format, or no longer matches the declarations.
  const RULE_ARTIFACT: &'static str = "externs.artifact";

  /// Judge an artifact against the declarations it is supposed to describe.
  ///
  /// The declarations are checked content here just as the artifact is: a malformed tree, a
  /// declaration this parser cannot read a contract from, or a name declared twice is a finding, and
  /// only an unreadable input - a refused root, a source the filesystem will not hand over - stays an
  /// execution failure.
  fn check_artifact(
    context: &mut CommandContext,
    declarations_root: &Path,
    path: &Path,
    format: ExternFormat,
    line_endings: Option<LineEndings>,
  ) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let started_at: Instant = Instant::now();

    let (externs, finding): (usize, Option<Finding>) =
      match ExternManifestParser::new().parse_directory(declarations_root) {
        Err(error @ XrfError::Io { .. }) => return Err(error.into()),
        // Declarations that never parsed exported nothing, and the count says so.
        Err(error) => (
          0,
          Some(Self::finding(Self::RULE_DECLARATIONS, declarations_root, &error)),
        ),
        Ok(parsed) => {
          let externs: usize = parsed.manifest.exports.len();

          match parsed.manifest.verify_artifact(path, format, line_endings) {
            Ok(()) => (externs, None),
            Err(error @ XrfError::Io { .. }) => return Err(error.into()),
            Err(error) => (externs, Some(Self::finding(Self::RULE_ARTIFACT, path, &error))),
          }
        }
      };
    let findings: Vec<Finding> = finding.into_iter().collect();
    let report: Report = Report::new(vec![CheckReport::new(
      CheckId::of_static(Self::CHECK),
      Status::from_is_valid(findings.is_empty()),
      Some(started_at.elapsed()),
      findings,
    )]);

    ReportVerdict::print_findings(&output, &report);

    // Deposited before the verdict becomes an outcome, so a failing check still reports what it found.
    context.set_result(|| ExternsCheckReport::new(declarations_root, path, format, externs, report.clone()))?;

    ReportVerdict::conclude(
      &output,
      &report,
      &format!("Extern artifact '{}' against {externs} declarations", format_path(path)),
    )
  }

  /// What judging the declarations or the artifact at `subject` found.
  fn finding(rule: &'static str, subject: &Path, error: &XrfError) -> Finding {
    Finding::new(
      RuleId::of_static(rule),
      Some(xrf_utils::to_portable_path_string(subject)),
      error.to_string(),
    )
  }

  /// The format `--format` names, or the one the artifact's extension says.
  fn resolve_format(matches: &ArgMatches, path: &Path) -> Result<ExternFormat, XrfError> {
    match matches.get_one::<String>("format") {
      Some(value) => ExternFormat::from_str(value),
      None => ExternFormat::from_extension(path),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use clap::ArgMatches;
  use xrf_export::ExternFormat;

  use super::ExportCommand;
  use crate::core::command::GenericCommand;

  fn parse(arguments: &[&str]) -> ArgMatches {
    ExportCommand.init().try_get_matches_from(arguments).unwrap()
  }

  /// Writing and checking name the artifact the same way, so both read its format off the extension.
  #[test]
  fn infers_the_format_from_the_extension_in_both_modes() {
    for arguments in [
      ["export", "-p", "declarations", "-d", "extern.xml"].as_slice(),
      ["export", "-p", "declarations", "-d", "extern.xml", "--check"].as_slice(),
    ] {
      assert_eq!(
        ExportCommand::resolve_format(&parse(arguments), &PathBuf::from("extern.xml")).unwrap(),
        ExternFormat::Xml
      );
    }
  }

  #[test]
  fn takes_an_explicit_format_over_the_extension() {
    let matches: ArgMatches = parse(&["export", "-p", "declarations", "-d", "extern.txt", "--format", "json"]);

    assert_eq!(
      ExportCommand::resolve_format(&matches, &PathBuf::from("extern.txt")).unwrap(),
      ExternFormat::Json
    );
  }
}
