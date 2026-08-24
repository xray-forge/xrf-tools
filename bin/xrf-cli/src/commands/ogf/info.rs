use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{OgfChunksProcessor, OgfFile, XRayByteOrder};
use xrf_output::OutputOptions;

use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  /// Create command for printing ogf file info.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to print information about provided ogf file")
      .arg(
        Arg::new("path")
          .help("Path to ogf file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("silent")
          .help("Disable any logging")
          .short('s')
          .long("silent")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Turn on verbose logging")
          .short('v')
          .long("verbose")
          .action(ArgAction::SetTrue),
      )
  }

  /// Print information about ogf file.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    xrf_output::info!(output, "Read ogf file {}", path.display());

    let ogf_file: Box<OgfFile> = Box::new(OgfFile::read_from_path::<XRayByteOrder, _>(path)?);

    xrf_output::info!(output, "Ogf file information");

    xrf_output::info!(
      output,
      "Version: {}, model_type: {}, shader_id: {}, {:?} - {:?}",
      ogf_file.header.version,
      ogf_file.header.model_type,
      ogf_file.header.shader_id,
      ogf_file.header.bounding_box,
      ogf_file.header.bounding_sphere
    );

    xrf_output::info!(output, "Boundaries box: {:?}", ogf_file.header.bounding_box);
    xrf_output::info!(output, "Boundaries sphere: {:?}", ogf_file.header.bounding_sphere);

    if let Some(texture) = &ogf_file.texture {
      xrf_output::info!(output, "Texture name: {}", texture.texture_name);
      xrf_output::info!(output, "Shader name: {}", texture.shader_name);
    }

    if let Some(description) = &ogf_file.description {
      xrf_output::info!(output, "Description: {:?}", description);
    }

    if let Some(bones) = &ogf_file.bones {
      xrf_output::info!(output, "Bones: {}", bones.bones.len());

      for (index, bone) in bones.bones.iter().enumerate() {
        xrf_output::info!(output, "[{}] name: {}", index, bone.name);
        xrf_output::info!(output, "[{}] parent: {}", index, bone.parent);
      }
    }

    if let Some(kinematics) = &ogf_file.kinematics {
      xrf_output::info!(output, "Motion refs: {:?}", kinematics.motion_refs);
    }

    if let Some(swi_data) = &ogf_file.swi_data {
      xrf_output::info!(output, "Progressive lods: {}", swi_data.windows.len());

      for (index, window) in swi_data.windows.iter().enumerate() {
        xrf_output::verbose!(
          output,
          "[{}] lod offset: {}, tris: {}, verts: {}",
          index,
          window.offset,
          window.num_tris,
          window.num_verts
        );
      }
    }

    match OgfChunksProcessor::find_unknown_chunk_ids::<XRayByteOrder, _>(path) {
      Ok(unknown) if !unknown.is_empty() => {
        xrf_output::info!(output, "Unparsed chunk ids: {:?}", unknown);
      }
      Ok(_) => xrf_output::verbose!(output, "Unparsed chunk ids: none"),
      Err(error) => xrf_output::warning!(output, "Could not survey chunks: {}", error),
    }

    if let Some(children) = &ogf_file.children {
      xrf_output::info!(output, "OGF children ({}):", children.nested.len());

      for (index, child) in children.nested.iter().enumerate() {
        if let Some(texture) = &child.texture {
          xrf_output::info!(output, "[{}] texture name: {}", index, texture.texture_name);
          xrf_output::info!(output, "[{}] shader name: {}", index, texture.shader_name);
        }

        // A child is a full visual, so progressive lods live here rather than on the root.
        if let Some(swi_data) = &child.swi_data {
          xrf_output::info!(output, "[{}] progressive lods: {}", index, swi_data.windows.len());
        }
      }
    }

    Ok(())
  }
}
