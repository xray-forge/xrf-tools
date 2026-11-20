use std::path::PathBuf;

use clap::{ArgMatches, Command};
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_thm::{
  ThmBumpChunk, ThmDetailChunk, ThmFile, ThmGroupFile, ThmKind, ThmMaterialChunk, ThmObjectFile, ThmSoundFile,
  ThmTextureFlag, ThmTextureParamChunk, ThmThumbnailChunk,
};
use xrf_utils::format_path;

use super::report::ThmInfoReport;
use crate::core::arguments::InputArgument;
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};

#[derive(Default)]
pub(crate) struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn input(&self) -> CommandInput {
    CommandInput::path("Path to thm file")
  }

  fn declare(&self, command: Command) -> Command {
    command.about("Print what a THM file holds, every field of a texture descriptor by the SDK's own names")
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = InputArgument::of(matches);
    let output: OutputOptions = context.get_output().clone();
    let bytes: Vec<u8> = std::fs::read(path)?;

    xrf_output::info!(output, "Read thm file {}", format_path(path));

    let report: ThmInfoReport = match ThmKind::detect::<XRayByteOrder>(bytes.clone())? {
      Some(ThmKind::Texture) => ThmInfoReport::Texture(ThmFile::read_from_bytes::<XRayByteOrder>(bytes)?),
      Some(ThmKind::Object) => ThmInfoReport::Object(ThmObjectFile::read_from_bytes::<XRayByteOrder>(bytes)?),
      Some(ThmKind::Group) => ThmInfoReport::Group(ThmGroupFile::read_from_bytes::<XRayByteOrder>(bytes)?),
      Some(ThmKind::Sound) => ThmInfoReport::Sound(ThmSoundFile::read_from_bytes::<XRayByteOrder>(bytes)?),
      None => {
        return Err(
          XrfError::new_invalid_error(format!(
            "{} declares no version and type pair any SDK thumbnail reader accepts",
            format_path(path)
          ))
          .into(),
        );
      }
    };

    match &report {
      ThmInfoReport::Texture(file) => Self::print_texture(&output, file),
      ThmInfoReport::Object(file) => Self::print_object(&output, file),
      ThmInfoReport::Group(file) => Self::print_group(&output, file),
      ThmInfoReport::Sound(file) => Self::print_sound(&output, file),
    }

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl InfoCommand {
  /// Prints a texture descriptor field by field, each value by the name the SDK's editor shows it under.
  fn print_texture(output: &OutputOptions, file: &ThmFile) {
    let param: ThmTextureParamChunk = file.texture_param.unwrap_or_default();
    let detail: ThmDetailChunk = file.detail.clone().unwrap_or_default();
    let material: ThmMaterialChunk = file.material.unwrap_or_default();
    let bump: ThmBumpChunk = file.bump.clone().unwrap_or_default();
    let mut flags: Vec<String> = param
      .flags
      .named()
      .map(|flag| ThmTextureFlag::label(flag).to_owned())
      .collect();

    if param.flags.unnamed() != 0 {
      flags.push(format!("unnamed {:#010x}", param.flags.unnamed()));
    }

    xrf_output::info!(
      output,
      "Kind: texture descriptor, version {}",
      file
        .version
        .map_or(String::from("none"), |version| format!("{version:#06x}"))
    );
    xrf_output::info!(output, "Type: {}", file.texture_type().label());
    xrf_output::info!(output, "Format: {}", param.format.label());
    xrf_output::info!(output, "Size: {} x {}", param.width, param.height);
    xrf_output::info!(output, "Mip filter: {}", param.mip_filter.label());
    xrf_output::info!(
      output,
      "Flags: {}",
      if flags.is_empty() {
        String::from("none")
      } else {
        flags.join(", ")
      }
    );
    xrf_output::info!(output, "Border colour: {:08X}", param.border_color);
    xrf_output::info!(
      output,
      "Fade: colour {:08X}, amount {}%, delay {}",
      param.fade_color,
      param.fade_amount,
      file.fade_delay.unwrap_or_default()
    );
    xrf_output::info!(output, "Detail: '{}', scale {}", detail.name, detail.scale);
    xrf_output::info!(
      output,
      "Material: {}, weight {}",
      material.material.label(),
      material.weight
    );
    xrf_output::info!(
      output,
      "Bump: {} '{}', virtual height {}",
      bump.mode.label(),
      bump.name,
      bump.virtual_height
    );
    xrf_output::info!(
      output,
      "External normal map: '{}'",
      file.ext_normal_map_name.as_deref().unwrap_or_default()
    );

    if let Some(thumbnail) = &file.thumbnail {
      Self::print_preview(output, thumbnail);
    }

    if !file.extra.is_empty() {
      xrf_output::info!(output, "Chunks kept unread: {}", file.extra.len());
    }
  }

  /// Prints an object's thumbnail: its size and how its preview is stored.
  fn print_object(output: &OutputOptions, file: &ThmObjectFile) {
    xrf_output::info!(output, "Kind: object thumbnail");
    xrf_output::info!(output, "Faces: {}, vertices: {}", file.face_count, file.vertex_count);
    Self::print_preview(output, &file.thumbnail);
  }

  /// Prints a group's thumbnail: the objects it gathers, one a line, and how its preview is stored.
  fn print_group(output: &OutputOptions, file: &ThmGroupFile) {
    xrf_output::info!(output, "Kind: group thumbnail");
    xrf_output::info!(output, "Objects: {}", file.objects.len());

    for object in &file.objects {
      xrf_output::info!(output, "  {}", object);
    }

    Self::print_preview(output, &file.thumbnail);
  }

  /// Prints a sound's thumbnail: what the SDK encodes the game sound with, the fallbacks of an older thumbnail applied.
  fn print_sound(output: &OutputOptions, file: &ThmSoundFile) {
    xrf_output::info!(output, "Kind: sound thumbnail");
    xrf_output::info!(output, "Quality: {}", file.quality);
    xrf_output::info!(output, "Distance: {} to {} m", file.min_distance, file.max_distance);
    xrf_output::info!(output, "AI distance: {} m", file.get_max_ai_distance());
    xrf_output::info!(output, "Base volume: {}", file.get_base_volume());
    xrf_output::info!(output, "Game type: {:#010x}", file.game_type);
  }

  /// Prints how a preview is stored; its pixels are the report's to carry, not the console's.
  fn print_preview(output: &OutputOptions, thumbnail: &ThmThumbnailChunk) {
    xrf_output::info!(
      output,
      "Preview: {} bytes{}",
      thumbnail.data.len(),
      if thumbnail.is_compressed { ", compressed" } else { "" }
    );
  }
}
