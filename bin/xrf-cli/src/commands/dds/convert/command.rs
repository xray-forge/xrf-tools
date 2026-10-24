use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_dds::{DdsEncodeAttempt, DdsEncodeCandidate, DdsFile, DdsMipChain, DdsMipmaps, Quality, RgbaImage};
use xrf_error::XrfResult;
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::DdsConvertReport;
use crate::commands::dds::dds_encode_arguments::DdsEncodeArguments;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct ConvertCommand;

impl GenericCommand for ConvertCommand {
  fn operation(&self) -> &'static str {
    "convert"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Re-encode a DDS file into another format, with its mip chain rebuilt")
      .arg(
        Arg::new("path")
          .help("Path of the dds file to read")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path of the dds file to write")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("format")
          .help("Format to write, of the five worth offering for an X-Ray texture")
          .long("format")
          .required(true)
          .value_parser(DdsEncodeCandidate::NAMES),
      );

    DdsEncodeArguments::declare(command, DdsEncodeArguments::CONVERT_MIP_FILTER)
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
    let source: &PathBuf = matches.get_one("path").expect("Expected valid source path");
    let destination: &PathBuf = matches.get_one("dest").expect("Expected valid destination path");
    let candidate: DdsEncodeCandidate = Self::get_candidate(matches)?;
    let quality: Quality = DdsEncodeArguments::quality_of(matches)?;
    let output: OutputOptions = context.get_output().clone();

    let base: RgbaImage = DdsFile::read_from_path(source)?.decode_rgba(0)?;
    let chain: DdsMipChain = DdsMipChain::build(&base, Self::get_mipmaps(matches)?)?;

    let written: DdsEncodeAttempt = DdsEncodeAttempt::measure(&chain, candidate, quality)?;

    written.file.write_to_path(destination)?;

    // The other four are measured against the same levels and thrown away; nothing but their figures is kept.
    let compared: Vec<DdsEncodeAttempt> = if matches.get_flag("compare") {
      DdsEncodeCandidate::ALL
        .into_iter()
        .filter(|it| *it != candidate)
        .map(|it| DdsEncodeAttempt::measure(&chain, it, quality))
        .collect::<XrfResult<Vec<DdsEncodeAttempt>>>()?
    } else {
      Vec::new()
    };

    xrf_output::success!(
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
  fn get_candidate(matches: &ArgMatches) -> XrfResult<DdsEncodeCandidate> {
    DdsEncodeCandidate::try_from(
      matches
        .get_one::<String>("format")
        .map(String::as_str)
        .unwrap_or_default(),
    )
  }

  fn get_mipmaps(matches: &ArgMatches) -> XrfResult<DdsMipmaps> {
    if matches.get_flag("no-mipmaps") {
      return Ok(DdsMipmaps::Disabled);
    }

    Ok(DdsMipmaps::Filtered(DdsEncodeArguments::mip_filter_of(matches)?))
  }
}
