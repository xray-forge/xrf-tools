use std::fs;
use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_ogf::{OgfChunksProcessor, OgfFile, OgfHeaderChunk};
use xrf_output::OutputOptions;
use xrf_spawn::XRayByteOrder;
use xrf_utils::format_path;

use super::report::OgfInfoReport;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn declare(&self, command: Command) -> Command {
    command.about("Print what an OGF visual holds").arg(
      Arg::new("path")
        .help("Path to the OGF file")
        .short('p')
        .long("path")
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Read ogf file {}", format_path(path));

    // Read once: the visual and the survey of the chunks it does not understand are two views of the same bytes.
    let bytes: Vec<u8> = fs::read(path).map_err(|error| {
      XrfError::new_not_found_error(format!("OGF file was not read: {}, error: {error}", format_path(path)))
    })?;
    let ogf_file: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;
    let header: &OgfHeaderChunk = &ogf_file.header;

    xrf_output::info!(
      output,
      "Version: {}, model type: {}, shader id: {}",
      header.version,
      header.model_type,
      header.shader_id
    );

    xrf_output::info!(
      output,
      "Bounding box: [{}, {}, {}] to [{}, {}, {}], sphere: [{}, {}, {}] radius {}",
      header.bounding_box.min.x,
      header.bounding_box.min.y,
      header.bounding_box.min.z,
      header.bounding_box.max.x,
      header.bounding_box.max.y,
      header.bounding_box.max.z,
      header.bounding_sphere.position.x,
      header.bounding_sphere.position.y,
      header.bounding_sphere.position.z,
      header.bounding_sphere.radius
    );

    if let Some(texture) = &ogf_file.texture {
      xrf_output::info!(output, "Texture name: {}", texture.texture_name);
      xrf_output::info!(output, "Shader name: {}", texture.shader_name);
    }

    if let Some(description) = &ogf_file.description {
      xrf_output::info!(
        output,
        "Source file: {}, convertor: {}, creator: {}, editor: {}",
        description.source_file,
        description.convertor,
        description.creator,
        description.editor
      );
    }

    if let Some(bones) = &ogf_file.bones {
      xrf_output::info!(output, "Bones: {}", bones.bones.len());

      for (index, bone) in bones.bones.iter().enumerate() {
        xrf_output::info!(output, "[{}] name: {}", index, bone.name);
        xrf_output::info!(output, "[{}] parent: {}", index, bone.parent);
      }
    }

    if let Some(kinematics) = &ogf_file.kinematics {
      xrf_output::info!(output, "Motion refs: {}", kinematics.motion_refs.join(", "));
    }

    if let Some(residue) = &ogf_file.residue {
      let ignored: usize = residue.bytes.len()
        + ogf_file
          .kinematics
          .as_ref()
          .map_or(0, |kinematics| kinematics.trailing.len());

      xrf_output::info!(
        output,
        "Residue: {} bytes from offset {} the engine never reads, {}",
        ignored,
        residue.position,
        residue.cause.as_str()
      );
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
    let unknown_chunks: Option<Vec<u32>> = match OgfChunksProcessor::find_unknown_chunk_ids::<XRayByteOrder>(bytes) {
      Ok(unknown) if !unknown.is_empty() => {
        xrf_output::info!(
          output,
          "Unparsed chunk ids: {}",
          unknown.iter().map(u32::to_string).collect::<Vec<String>>().join(", ")
        );

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
