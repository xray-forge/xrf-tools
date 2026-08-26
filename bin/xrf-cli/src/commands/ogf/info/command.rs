use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_db::{OgfChunksProcessor, OgfFile, XRayByteOrder};
use xrf_output::OutputOptions;

use super::report::OgfInfoReport;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

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
  }

  /// Print information about ogf file.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = context.get_output().clone();

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

    // Kept for the report: which chunks went unparsed is how an unsupported visual is told apart
    // from an empty one, and a survey that could not run says nothing rather than "none".
    let unknown_chunks: Option<Vec<u32>> = match OgfChunksProcessor::find_unknown_chunk_ids::<XRayByteOrder, _>(path) {
      Ok(unknown) if !unknown.is_empty() => {
        xrf_output::info!(output, "Unparsed chunk ids: {:?}", unknown);

        Some(unknown)
      }
      Ok(unknown) => {
        xrf_output::verbose!(output, "Unparsed chunk ids: none");

        Some(unknown)
      }
      Err(error) => {
        xrf_output::warning!(output, "Could not survey chunks: {}", error);

        None
      }
    };

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

    context.set_result(|| OgfInfoReport::new(&ogf_file, unknown_chunks))?;

    Ok(())
  }
}
