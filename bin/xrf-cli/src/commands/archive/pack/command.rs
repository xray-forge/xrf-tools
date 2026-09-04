use std::path::PathBuf;

use clap::parser::ValueSource;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_pack::{
  ArchivePackConfig, ArchivePackDirectory, ArchivePackHeaderEntry, ArchivePackMode, ArchivePackOptions,
  ArchivePackResult, ArchivePacker, ArchiveVolumeExtension, VOLUME_SIZE_MAX, VOLUME_SIZE_MIN,
};
use xrf_utils::format_path;

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

/// The options that name the selection rules directly, which a configuration file also carries.
const SELECTION_ARGUMENTS: [&str; 7] = [
  "include-file",
  "include-directory",
  "include-directory-shallow",
  "exclude-directory",
  "exclude-directory-shallow",
  "exclude-extension",
  "header",
];

#[derive(Default)]
pub struct PackCommand;

impl GenericCommand for PackCommand {
  fn operation(&self) -> &'static str {
    "pack"
  }

  /// Create command to pack a directory into archive volumes.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to pack provided directory into *.db archive volumes")
      .arg(
        Arg::new("path")
          .help("Path to the directory to pack, normally a gamedata root")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to folder for writing the volumes")
          .short('d')
          .long("dest")
          .default_value("packed")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("name")
          .help("Base name of the volumes, written as <name>.db0, <name>.db1 and so on")
          .short('n')
          .long("name")
          .default_value("gamedata")
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("config")
          .help("Path to a packing configuration describing what to include, as *.ltx or *.json")
          .long("config")
          .required(false)
          .conflicts_with_all(SELECTION_ARGUMENTS)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("include-file")
          .help("File to pack, named relative to --path, repeatable")
          .long("include-file")
          .required(false)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("include-directory")
          .help("Directory to pack with everything below it, relative to --path, repeatable")
          .long("include-directory")
          .required(false)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("include-directory-shallow")
          .help("Directory whose own files are packed while its subdirectories only get listed, repeatable")
          .long("include-directory-shallow")
          .required(false)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("exclude-directory")
          .help("Directory to leave out along with everything below it, repeatable")
          .long("exclude-directory")
          .required(false)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("exclude-directory-shallow")
          .help("Directory to leave out while its contents still pack, repeatable")
          .long("exclude-directory-shallow")
          .required(false)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("exclude-extension")
          .help("Extension pattern that keeps a file out, such as *.txt, repeatable")
          .long("exclude-extension")
          .required(false)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("header")
          .help("Header entry written into the archive as <key>=<value>, repeatable, replacing the default header")
          .long("header")
          .required(false)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("store")
          .help("Store every file instead of compressing what the engine expects compressed")
          .long("store")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("max-size")
          .help(format!(
            "Maximum volume size in megabytes, from {} to {}",
            VOLUME_SIZE_MIN / xrf_utils::BYTES_PER_MEGABYTE,
            VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
          ))
          .long("max-size")
          .required(false)
          .value_parser(value_parser!(u64).range(1..)),
      )
      .arg(
        Arg::new("oversized-volumes")
          .help(format!(
            "Let --max-size exceed {} MB, which only an engine fork that raised XRP_MAX_SIZE can mount",
            VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
          ))
          .long("oversized-volumes")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("xdb")
          .help("Write volumes with the xdb extension")
          .long("xdb")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("no-skip-list")
          .help("Keep editor and source leftovers the engine build normally drops")
          .long("no-skip-list")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("force")
          .help("Replace volumes of the same set the destination already holds")
          .short('f')
          .long("force")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Pack a directory into xray engine database archives.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: PathBuf = xrf_utils::to_absolute_path(
      matches
        .get_one::<PathBuf>("path")
        .expect("Expected valid source path to be provided"),
    )?;

    let destination: PathBuf = xrf_utils::to_absolute_path(
      matches
        .get_one::<PathBuf>("dest")
        .expect("Expected valid output path to be provided"),
    )?;

    let name: &String = matches
      .get_one::<_>("name")
      .expect("Expected valid archive name to be provided");

    let output: OutputOptions = context.get_output().clone();

    let mut config: ArchivePackConfig = ArchivePackConfig::new(&path, &destination, name);

    // One selection source or the other, never both: clap refuses `--config` beside a selection option, so whichever
    // is present supplies the whole selection. The run options below still layer over either.
    if let Some(path) = matches.get_one::<PathBuf>("config") {
      xrf_output::info!(output, "Pack config: {}", format_path(path));

      config = config.with_config_file(path)?;
    } else {
      config = Self::with_selection_arguments(config, matches)?;
    }

    if matches.get_flag("store") {
      config.mode = ArchivePackMode::Store;
    }

    if matches.get_flag("xdb") {
      config.volume_extension = ArchiveVolumeExtension::Xdb;
    }

    if matches.get_flag("no-skip-list") {
      config.is_with_skip_list = false;
    }

    // Before the size it lifts the bound for, which is the order `with_max_volume_size` reads them in.
    let is_oversized_allowed: bool = matches.get_flag("oversized-volumes");

    config = config.with_oversized_volumes(is_oversized_allowed);

    if matches.value_source("max-size") == Some(ValueSource::CommandLine)
      && let Some(size) = matches.get_one::<u64>("max-size")
    {
      // `--max-size` is given in megabytes, matching the `-max_size` unit of xrCompress.
      let size: u64 = xrf_utils::megabytes_to_bytes(*size);

      // Asked here as well as in the configuration, so the refusal can name the flag that lifts it. The
      // configuration refuses the same size regardless; this is the surface's own wording, not its own rule.
      if size > VOLUME_SIZE_MAX && !is_oversized_allowed {
        return Err(
          XrfError::new_invalid_error(format!(
            "Volume size {} is past the {} MB the engine mounts. Pass --oversized-volumes to publish \
             volumes only a fork that raised XRP_MAX_SIZE can open.",
            xrf_utils::format_bytes(size),
            VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
          ))
          .into(),
        );
      }

      config = config.with_max_volume_size(size)?;
    }

    xrf_output::info!(output, "Pack source: {}", format_path(&path));
    xrf_output::info!(output, "Pack destination: {}", format_path(&destination));

    if config.max_volume_size > VOLUME_SIZE_MAX {
      xrf_output::warning!(
        output,
        "Volumes are capped at {}, past the {} MB no unmodified engine mounts. These volumes load only in \
         a fork that raised XRP_MAX_SIZE.",
        xrf_utils::format_bytes(config.max_volume_size),
        VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
      );
    }

    // A headerless archive is not neutral to the engine: unless it is named `xdb`, the loader assumes it
    // is an encrypted Shadow of Chernobyl archive and decrypts it into nonsense.
    if config.header.is_none() && config.volume_extension != ArchiveVolumeExtension::Xdb {
      xrf_output::warning!(
        output,
        "No [header] section configured: the engine will read these volumes as encrypted ShoC archives. \
         Supply a header naming an entry_point, with --header or in --config, or pass --xdb."
      );
    }

    let is_forced: bool = matches.get_flag("force");

    // Asked here as well as inside the pack, so the refusal can name the flag that lifts it. Packing refuses the same
    // destination regardless; this is the surface's own wording, not its own rule.
    if !is_forced {
      let published: Vec<PathBuf> = ArchivePacker::list_published_volumes(&config)?;

      if !published.is_empty() {
        return Err(
          XrfError::new_invalid_error(format!(
            "Destination '{}' already holds {} volume(s) of '{}'. Packing replaces them and cannot put them back if it \
           fails partway: move them aside, or pass --force to overwrite them.",
            format_path(&destination),
            published.len(),
            config.single_volume_name()
          ))
          .into(),
        );
      }
    }

    // The packer says each decision itself, at verbose level, as it is made: a log of a run that stops is then
    // already naming the volume and the last entry it reached.
    let result: ArchivePackResult = ArchivePacker::pack_opt(
      &config,
      ArchivePackOptions::default()
        .with_output(output.clone())
        .with_force(is_forced),
    )?;

    for volume in &result.volumes {
      xrf_output::info!(output, "Wrote {}", format_path(volume));
    }

    xrf_output::success!(
      output,
      "Packed {} file(s) into {} volume(s) in {}",
      result.files_total,
      result.volumes.len(),
      xrf_utils::format_duration(result.duration),
    );

    xrf_output::info!(
      output,
      "Phases: {} collecting, {} writing, {} finalizing",
      xrf_utils::format_duration(result.collect_duration),
      xrf_utils::format_duration(result.write_duration),
      xrf_utils::format_duration(result.finalize_duration),
    );

    xrf_output::info!(
      output,
      "Summary: {} compressed, {} stored, {} aliased, {} skipped",
      result.files_compressed,
      result.files_stored,
      result.files_aliased,
      result.files_skipped,
    );

    let (size_source, size_written): (String, String) =
      xrf_utils::format_bytes_pair(result.size_source, result.size_written);

    xrf_output::info!(output, "Size: {size_source} source, {size_written} written");

    // Read off the result rather than divided here, so the terminal and the report cannot disagree on it.
    xrf_output::info!(output, "Speed: {}/s", xrf_utils::format_bytes(result.speed));

    context.set_result(|| &result)?;

    Ok(())
  }
}

impl PackCommand {
  /// Apply the selection rules named on the command line.
  ///
  /// The direct half of what a configuration file carries, for a caller that already holds the values and has no use
  /// for a file between them. `--config` refuses to appear beside these, so whichever mode is in play supplies the
  /// whole selection rather than layering over the other.
  ///
  /// Recursive and shallow entries of one kind arrive as two lists rather than interleaved, which the archive cannot
  /// tell apart: registration keys every row by engine name and sorts it, so the volume a run writes does not depend
  /// on the order its directories were named in.
  fn with_selection_arguments(mut config: ArchivePackConfig, matches: &ArgMatches) -> XrfResult<ArchivePackConfig> {
    if let Some(files) = matches.get_many::<String>("include-file") {
      config.include_files = files.cloned().collect();
    }

    if let Some(directories) = Self::collect_directories(matches, "include-directory", "include-directory-shallow") {
      config.include_directories = directories;
    }

    if let Some(directories) = Self::collect_directories(matches, "exclude-directory", "exclude-directory-shallow") {
      config.exclude_directories = directories;
    }

    if let Some(extensions) = matches.get_many::<String>("exclude-extension") {
      config.exclude_extensions = extensions.cloned().collect();
    }

    if let Some(entries) = matches.get_many::<String>("header") {
      let entries: Vec<ArchivePackHeaderEntry> = entries
        .map(|entry| Self::parse_header_entry(entry))
        .collect::<XrfResult<_>>()?;

      config = config.with_header_entries(&entries);
    }

    Ok(config)
  }

  /// Read one kind of directory rule from its recursive and shallow options, or nothing when neither was given.
  ///
  /// Answers `None` rather than an empty list so a run that names no directories leaves the configuration's own alone,
  /// the way every other field here does. The two are the same today, since a fresh configuration selects nothing, and
  /// would stop being the same the moment a default did.
  ///
  /// The flag stores `is_recursive`, which the packer reads differently on each side: an included directory takes
  /// everything below it, while an excluded one covers everything below it rather than only the name it gives. The
  /// two options exist so neither meaning has to be spelled into a value.
  fn collect_directories(matches: &ArgMatches, recursive: &str, shallow: &str) -> Option<Vec<ArchivePackDirectory>> {
    let sources: [(&str, bool); 2] = [(recursive, true), (shallow, false)];

    if sources
      .iter()
      .all(|(argument, _)| matches.get_many::<String>(argument).is_none())
    {
      return None;
    }

    Some(
      sources
        .into_iter()
        .flat_map(|(argument, is_recursive)| {
          matches
            .get_many::<String>(argument)
            .into_iter()
            .flatten()
            .map(move |path| ArchivePackDirectory {
              path: path.clone(),
              is_recursive,
            })
        })
        .collect(),
    )
  }

  /// Split one `--header <key>=<value>` into the pair the archive stores.
  ///
  /// Split at the first `=` so a value may hold its own, which the engine's own header does for quoted text. An entry
  /// naming no key is refused rather than written, since the engine would read the line and find nothing addressed.
  fn parse_header_entry(entry: &str) -> XrfResult<ArchivePackHeaderEntry> {
    let (key, value) = entry.split_once('=').ok_or_else(|| {
      XrfError::new_invalid_error(format!(
        "Header entry '{entry}' is not a <key>=<value> pair. Write it as --header auto_load=true."
      ))
    })?;

    let key: &str = key.trim();

    if key.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Header entry '{entry}' names no key. Write it as --header auto_load=true."
      )));
    }

    Ok(ArchivePackHeaderEntry {
      key: key.to_string(),
      value: value.trim().to_string(),
    })
  }
}
