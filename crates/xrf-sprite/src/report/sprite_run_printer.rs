use xrf_dds::ImageFormat;
use xrf_job::JobOutcome;
use xrf_output::OutputOptions;
use xrf_texture::ImageFileReshape;
use xrf_utils::format_path;

use crate::description::{DescriptionSpriteOptions, DescriptionSpriteResult};
use crate::equipment::EquipmentSpritePackResult;
use crate::report::SpriteNotice;

/// What a person reads of a sprite run: the notices a run hands out and the counts it answers, kept in one place so every
/// surface running one says the same thing about the same run.
///
/// The runs themselves print nothing; this renders what they hand back.
pub struct SpriteRunPrinter;

impl SpriteRunPrinter {
  /// Prints a notice as the warning it is.
  pub fn print_notice(notice: &SpriteNotice, output: &OutputOptions) {
    xrf_output::warning!(output, "{}", Self::format_notice(notice));
  }

  /// A notice as one line of text, for a surface that keeps its own log.
  pub fn format_notice(notice: &SpriteNotice) -> String {
    match notice {
      SpriteNotice::IconUnreadable { path, section, reason } => {
        format!("Skip icon {} / '{section}', reason: {reason}", format_path(path))
      }
      SpriteNotice::SlotConflict {
        x,
        y,
        previous,
        section,
      } => format!(
        "Slot {x}:{y} is claimed by '{previous}' and '{section}' with different icons, only the last one packed survives"
      ),
      SpriteNotice::IconOutsideSheet { section } => {
        format!("Skip for possible section: '{section}' - icon is out of source file bonds")
      }
      SpriteNotice::SpriteUnreadable {
        id,
        file,
        sheet,
        reason,
      } => format!(
        "Failed to read texture dds {id} for {file} ({}): {reason}",
        format_path(sheet)
      ),
      SpriteNotice::SpriteOutsideSheet {
        id,
        max_x,
        max_y,
        width,
        height,
        sheet,
      } => format!(
        "[WARN] - exceeding sprite size '{id}' (x:{max_x}, y:{max_y}) ({width}x{height} - {})",
        format_path(sheet)
      ),
      SpriteNotice::SheetUnreadable { path, reason } => {
        format!("Skip file {}, not able to read: {reason}", format_path(path))
      }
      SpriteNotice::SheetReshaped {
        path,
        width,
        height,
        mipmap_levels,
        replaced,
      } => match replaced {
        ImageFileReshape::Unreadable { reason } => {
          format!(
            "Cannot compare shape against replaced file {}: {reason}",
            format_path(path)
          )
        }
        ImageFileReshape::Reshaped {
          width: replaced_width,
          height: replaced_height,
          mipmap_levels: replaced_mipmap_levels,
        } => format!(
          "Replacing {} of {replaced_width}x{replaced_height} with {replaced_mipmap_levels} mipmap levels by \
           {width}x{height} with {mipmap_levels} mipmap levels",
          format_path(path)
        ),
      },
      SpriteNotice::DescriptionUnparsable { path, reason } => {
        format!("Error parsing XML file: {} - {reason}", format_path(path))
      }
      SpriteNotice::DescriptionWithoutWindow { path } => {
        format!("Got no 'w' tag for file '{}'", format_path(path))
      }
      SpriteNotice::FileWithoutName => String::from("Invalid file node supplied without name attribute"),
      SpriteNotice::FileWithoutTextures { name, path } => {
        format!(
          "Skip definitions node \"{name}\" without textures (in {})",
          format_path(path)
        )
      }
      SpriteNotice::TextureNodeIncomplete { id, attributes } => format!(
        "Skip texture node: {} ({})",
        id.as_deref().unwrap_or("unknown"),
        attributes
          .iter()
          .map(|(name, value)| format!("{name}={value}"))
          .collect::<Vec<String>>()
          .join(",")
      ),
    }
  }

  /// How many icons a completed equipment pack drew, and in which format; a cancelled one wrote nothing to count.
  pub fn print_equipment_pack(result: &EquipmentSpritePackResult, format: ImageFormat, output: &OutputOptions) {
    if result.outcome == JobOutcome::Completed {
      xrf_output::info!(output, "Packed {} icons in {format} format", result.packed_count);
    }
  }

  /// How many icons an equipment unpack wrote.
  pub fn print_equipment_unpack(unpacked: u32, output: &OutputOptions) {
    xrf_output::info!(output, "Unpacked {unpacked} icons");
  }

  /// Which description a pack read and how many of its files it packed.
  pub fn print_description_pack(
    options: &DescriptionSpriteOptions,
    result: &DescriptionSpriteResult,
    output: &OutputOptions,
  ) {
    Self::print_description_source(options, output);

    xrf_output::info!(output, "Packing for {} files", result.files + result.skipped_files);
    xrf_output::info!(output, "Packed {} files", result.files);
  }

  /// Which description an unpack read and how many of its files it unpacked.
  pub fn print_description_unpack(
    options: &DescriptionSpriteOptions,
    result: &DescriptionSpriteResult,
    output: &OutputOptions,
  ) {
    Self::print_description_source(options, output);

    xrf_output::info!(output, "Unpacking for {} files", result.files + result.skipped_files);
    xrf_output::info!(output, "Unpacked {} files", result.files);
  }

  /// Names the directory a run read its descriptions from; a single description file is already named by the caller.
  fn print_description_source(options: &DescriptionSpriteOptions, output: &OutputOptions) {
    if options.description.is_dir() {
      xrf_output::info!(
        output,
        "Check texture descriptions from dir: {}",
        format_path(&options.description)
      );
    }
  }
}
