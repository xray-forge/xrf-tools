use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_dds::{
  DdsEncodeAttempt, DdsEncodeCandidate, DdsFile, DdsMipChain, DdsMipFilter, DdsMipmaps, Quality, RgbaImage,
};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::DdsConvertReport;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

/// How hard the encoder works, which is not a knob a command line needs: a conversion writes a file somebody keeps.
const QUALITY: Quality = Quality::Slow;

#[derive(Default)]
pub struct ConvertCommand;

impl GenericCommand for ConvertCommand {
  fn operation(&self) -> &'static str {
    "convert"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to re-encode a dds file into another format, with its mip chain rebuilt")
      .arg(
        Arg::new("source")
          .help("Path of the dds file to read")
          .value_name("SOURCE")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("destination")
          .help("Path of the dds file to write")
          .value_name("DESTINATION")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("format")
          .help("Format to write, of the five worth offering for an X-Ray texture")
          .long("format")
          .required(true)
          .value_parser(["bc1", "bc2", "bc3", "bc7", "rgba8"]),
      )
      .arg(
        Arg::new("mip-filter")
          .help("Kernel the mip chain is reduced with, from the X-Ray converter's own family")
          .long("mip-filter")
          .default_value("kaiser")
          .value_parser(Self::filter_names()),
      )
      .arg(
        Arg::new("no-mipmaps")
          .help("Write only the base level, for a texture the engine never minifies")
          .long("no-mipmaps")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("compare")
          .help("Also report what every other candidate format would have cost, which is four more encodes")
          .long("compare")
          .action(ArgAction::SetTrue),
      )
  }

  /// Re-encode one texture, and say what the choice cost.
  ///
  /// The source is decoded once and reduced once; every candidate is then encoded from those same levels, so a
  /// comparison weighs formats rather than weighing one format against a differently built chain.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let source: &PathBuf = matches.get_one("source").expect("Expected valid source path");
    let destination: &PathBuf = matches.get_one("destination").expect("Expected valid destination path");
    let candidate: DdsEncodeCandidate = Self::get_candidate(matches)?;
    let output: OutputOptions = context.get_output().clone();

    let base: RgbaImage = DdsFile::read_from_path(source)?.decode_rgba(0)?;
    let chain: DdsMipChain = DdsMipChain::build(&base, Self::get_mipmaps(matches)?)?;

    let written: DdsEncodeAttempt = DdsEncodeAttempt::measure(&chain, candidate, QUALITY)?;

    written.file.write_to_path(destination)?;

    // The other four are measured against the same levels and thrown away; nothing but their figures is kept.
    let compared: Vec<DdsEncodeAttempt> = if matches.get_flag("compare") {
      DdsEncodeCandidate::ALL
        .into_iter()
        .filter(|it| *it != candidate)
        .map(|it| DdsEncodeAttempt::measure(&chain, it, QUALITY))
        .collect::<XrfResult<Vec<DdsEncodeAttempt>>>()?
    } else {
      Vec::new()
    };

    xrf_output::info!(
      output,
      "Converted {} to {} as {}, {} levels, {} bytes",
      format_path(source),
      format_path(destination),
      candidate.label(),
      chain.levels().len(),
      written.file_bytes
    );

    context.set_result(|| DdsConvertReport::new(source, destination, &written, &compared))?;

    Ok(())
  }
}

impl ConvertCommand {
  /// The filter names the command accepts, taken from the family itself so the two cannot drift.
  fn filter_names() -> Vec<String> {
    DdsMipFilter::NAMED
      .iter()
      .map(|filter| filter.label().to_lowercase())
      .collect()
  }

  fn get_candidate(matches: &ArgMatches) -> XrfResult<DdsEncodeCandidate> {
    match matches.get_one::<String>("format").map(String::as_str) {
      Some("bc1") => Ok(DdsEncodeCandidate::Bc1),
      Some("bc2") => Ok(DdsEncodeCandidate::Bc2),
      Some("bc3") => Ok(DdsEncodeCandidate::Bc3),
      Some("bc7") => Ok(DdsEncodeCandidate::Bc7),
      Some("rgba8") => Ok(DdsEncodeCandidate::Rgba8),
      other => Err(XrfError::new_invalid_error(format!(
        "Unexpected texture format '{}'",
        other.unwrap_or_default()
      ))),
    }
  }

  fn get_mipmaps(matches: &ArgMatches) -> XrfResult<DdsMipmaps> {
    if matches.get_flag("no-mipmaps") {
      return Ok(DdsMipmaps::Disabled);
    }

    let name: &str = matches
      .get_one::<String>("mip-filter")
      .map(String::as_str)
      .unwrap_or_default();

    DdsMipFilter::NAMED
      .into_iter()
      .find(|filter| filter.label().eq_ignore_ascii_case(name))
      .map(DdsMipmaps::Filtered)
      .ok_or_else(|| XrfError::new_invalid_error(format!("Unexpected mip filter '{name}'")))
  }
}
