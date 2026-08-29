use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_export::{
  ExternFormat, ExternManifest, ExternManifestParser, LineEndings, ParsedExternManifest, normalize_line_endings,
  render_extern_manifest,
};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::ExternsExportReport;
use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};

/// Generate or verify a stable extern manifest from TypeScript declarations.
#[derive(Default)]
pub struct ExportCommand;

impl GenericCommand for ExportCommand {
  fn operation(&self) -> &'static str {
    "export"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Export TypeScript extern declarations as JSON, XML, or HTML")
      .arg(
        Arg::new("declarations-root")
          .help("Root directory containing TypeScript declaration sources")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("format")
          .help("Output format; required with --output and inferred from --check when omitted")
          .long("format")
          .value_parser(["json", "xml", "html"]),
      )
      .arg(
        Arg::new("output")
          .help("Artifact to create or replace")
          .long("output")
          .conflicts_with("check")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("check")
          .help("Existing artifact to verify without writing")
          .long("check")
          .conflicts_with("output")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("line-endings")
          .help("Override generated line endings")
          .long("line-endings")
          .value_parser(["lf", "crlf"]),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let declarations_root: &PathBuf = matches
      .get_one("declarations-root")
      .expect("Expected declarations root");
    let output_dir: Option<&PathBuf> = matches.get_one("output");
    let check: Option<&PathBuf> = matches.get_one("check");

    if output_dir.is_none() && check.is_none() {
      return Err(XrfError::new_invalid_error("Specify exactly one of --output or --check.").into());
    }

    let line_endings: Option<LineEndings> = matches
      .get_one::<String>("line-endings")
      .map(|value: &String| LineEndings::from_str(value))
      .transpose()?;

    let output: OutputOptions = context.get_output().clone();
    let format: ExternFormat = Self::resolve_format(matches, output_dir, check)?;

    if let Some(path) = output_dir {
      // Writing judges nothing, so invalid declarations are simply work the command cannot do.
      let parsed: ParsedExternManifest = ExternManifestParser::new().parse_directory(declarations_root)?;
      let externs: usize = parsed.manifest.exports.len();
      let content: String = render_extern_manifest(&parsed.manifest, format, line_endings)?;

      Self::write_output(path, &content)?;

      xrf_output::info!(output, "Exported {externs} externs to '{}'.", format_path(path));

      return context.set_result(|| ExternsExportReport::written(declarations_root, path, format, externs));
    }

    let path: &PathBuf = check.expect("Checked path is required after validation");

    Self::check_artifact(context, declarations_root, path, format, line_endings)
  }
}

impl ExportCommand {
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

    let (externs, finding): (usize, Option<XrfError>) =
      match ExternManifestParser::new().parse_directory(declarations_root) {
        Err(error @ XrfError::Io { .. }) => return Err(error.into()),
        // Declarations that never parsed exported nothing, and the count says so.
        Err(error) => (0, Some(error)),
        Ok(parsed) => {
          let externs: usize = parsed.manifest.exports.len();

          match Self::verify_artifact(path, format, &parsed.manifest, line_endings) {
            Ok(()) => (externs, None),
            Err(error @ XrfError::Io { .. }) => return Err(error.into()),
            Err(error) => (externs, Some(error)),
          }
        }
      };

    let Some(error) = finding else {
      xrf_output::info!(
        output,
        "Extern artifact '{}' matches {externs} declarations.",
        format_path(path)
      );

      return context.set_result(|| ExternsExportReport::checked(declarations_root, path, format, externs, Vec::new()));
    };

    xrf_output::failure!(output, "{error}");

    // Deposited before the verdict becomes an outcome, so a failing check still reports what it found.
    context
      .set_result(|| ExternsExportReport::checked(declarations_root, path, format, externs, vec![error.to_string()]))?;

    Err(CommandError::new_check_failed(1))
  }

  fn resolve_format(
    matches: &ArgMatches,
    output: Option<&PathBuf>,
    check: Option<&PathBuf>,
  ) -> Result<ExternFormat, XrfError> {
    if let Some(value) = matches.get_one::<String>("format") {
      return ExternFormat::from_str(value);
    }

    if let Some(path) = check {
      return ExternFormat::from_extension(path);
    }

    let path: &PathBuf = output.expect("Output is required after validation");

    Err(XrfError::new_invalid_error(format!(
      "--format is required when writing '{}'.",
      format_path(path)
    )))
  }

  fn write_output(path: &Path, content: &str) -> Result<(), XrfError> {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;

    Ok(())
  }

  fn verify_artifact(
    path: &Path,
    format: ExternFormat,
    manifest: &ExternManifest,
    line_endings: Option<LineEndings>,
  ) -> Result<(), XrfError> {
    let existing: String = fs::read_to_string(path)?;

    match format {
      ExternFormat::Json => {
        let actual: ExternManifest = serde_json::from_str(&existing).map_err(|error| {
          XrfError::new_invalid_error(format!(
            "Cannot parse '{}' as an extern JSON manifest: {error}",
            format_path(path)
          ))
        })?;

        if actual != *manifest {
          return Err(XrfError::new_verify_error(format!(
            "Extern JSON artifact '{}' does not match the parsed declaration manifest.",
            format_path(path)
          )));
        }
      }

      ExternFormat::Xml | ExternFormat::Html => {
        let expected: String = render_extern_manifest(manifest, format, line_endings)?;

        if normalize_line_endings(&existing) != normalize_line_endings(&expected) {
          return Err(XrfError::new_verify_error(format!(
            "Extern {} artifact '{}' does not match freshly rendered output.",
            match format {
              ExternFormat::Xml => "XML",
              ExternFormat::Html => "HTML",
              ExternFormat::Json => unreachable!(),
            },
            format_path(path)
          )));
        }
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use clap::ArgMatches;
  use xrf_export::ExternFormat;

  use super::ExportCommand;
  use crate::core::generic_command::GenericCommand;

  #[test]
  fn infers_check_format_from_extension() {
    let matches: ArgMatches = ExportCommand::new()
      .init()
      .try_get_matches_from(["export", "declarations", "--check", "extern.xml"])
      .unwrap();
    let check: Option<&PathBuf> = matches.get_one("check");

    assert_eq!(
      ExportCommand::resolve_format(&matches, None, check).unwrap(),
      ExternFormat::Xml
    );
  }

  #[test]
  fn rejects_conflicting_output_modes() {
    assert!(
      ExportCommand::new()
        .init()
        .try_get_matches_from([
          "export",
          "declarations",
          "--output",
          "extern.json",
          "--check",
          "extern.json",
        ])
        .is_err()
    );
  }
}
