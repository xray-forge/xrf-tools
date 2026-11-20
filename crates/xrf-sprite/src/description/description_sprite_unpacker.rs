use std::path::PathBuf;
use std::time::Instant;

use image::{GenericImageView, RgbaImage};
use rayon::prelude::*;
use xrf_dds::{DdsFile, DdsMipmaps};
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_utils::{HostDestination, HostRelativePath, OutputFiles, format_path};

use crate::description::{
  DescriptionSpriteOptions, DescriptionSpriteResult, TextureFileDescriptor, XmlDescriptionCollection,
};
use crate::report::{SpriteListener, SpriteNotice};
use xrf_texture::ImageFileWriter;

pub struct DescriptionSpriteUnpacker {}

impl DescriptionSpriteUnpacker {
  /// Cuts the sprites out of the sheets the description declares, every one or those `options` names, and answers
  /// what was written, handing whatever it passed over to the options' listener.
  ///
  /// The sheets are cut in parallel, so their notices are handed out once every sheet is done, in selection order
  /// rather than in the order the workers finished; a run that fails hands out only what reading the description met.
  pub fn unpack_xml_descriptions(options: &DescriptionSpriteOptions) -> XrfResult<DescriptionSpriteResult> {
    let listener: Option<SpriteListener> = options.listener.clone();
    let mut on_notice = |notice: SpriteNotice| {
      if let Some(listener) = &listener {
        listener(&notice);
      }
    };
    let started_at: Instant = Instant::now();
    let description: XmlDescriptionCollection = XmlDescriptionCollection::get_descriptions(options, &mut on_notice)?;
    let selected: Vec<&TextureFileDescriptor> = description.select_files(options)?;

    // Every name is checked, and every sprite already there refused unforced, before the first sprite is written, so a
    // refusal leaves no partial output behind.
    let mut outputs: OutputFiles = OutputFiles::new(&options.output_path);

    for file in &selected {
      for path in file.to_sprite_paths()? {
        outputs = outputs.with_file(&path.to_string());
      }
    }

    outputs.assert_free(options.is_forced)?;

    let unpacking: xrf_job::JobScope = options.job.enter(
      crate::job_phases::SPRITE_PHASE_UNPACK_DESCRIPTIONS,
      Some(selected.len() as u64),
    );

    // Reduction keeps the indexed order, so the notices come out as the sheets were selected however wide the run is.
    let (result, notices): (DescriptionSpriteResult, Vec<SpriteNotice>) = selected
      .par_iter()
      .map(|file| {
        let mut notices: Vec<SpriteNotice> = Vec::new();
        let unpacked: DescriptionSpriteResult =
          Self::unpack_xml_description(options, file, &mut |notice| notices.push(notice))?;

        unpacking.advance();

        Ok::<(DescriptionSpriteResult, Vec<SpriteNotice>), XrfError>((unpacked, notices))
      })
      .try_reduce(
        || (DescriptionSpriteResult::default(), Vec::new()),
        |(left, mut left_notices), (right, right_notices)| {
          left_notices.extend(right_notices);

          Ok((left.merged(right), left_notices))
        },
      )?;

    notices.into_iter().for_each(on_notice);

    Ok(DescriptionSpriteResult {
      duration: started_at.elapsed(),
      ..result
    })
  }

  /// Cuts the sprites out of one described sheet, answering how many were written and how many did not fit; a sheet
  /// that cannot be read is passed over whole outside strict mode.
  pub(crate) fn unpack_xml_description(
    options: &DescriptionSpriteOptions,
    file: &TextureFileDescriptor,
    on_notice: &mut impl FnMut(SpriteNotice),
  ) -> XrfResult<DescriptionSpriteResult> {
    let relative_path: HostRelativePath = file.to_host_relative_path()?;
    let sprite_paths: Vec<HostRelativePath> = file.to_sprite_paths()?;
    let full_name: PathBuf = options
      .base
      .join(relative_path.with_extension(XrayExtension::Dds.as_str()));
    let destination: HostDestination = HostDestination::new(&options.output_path, options.link_policy);

    let dds: RgbaImage = match DdsFile::read_from_path(&full_name).and_then(|dds| dds.decode_rgba(0)) {
      Ok(dds) => dds,
      Err(_) if options.is_strict => {
        return Err(XrfError::new_texture_processing_error(format!(
          "Could not find file for texture unpacking: {}",
          format_path(&full_name)
        )));
      }
      Err(error) => {
        on_notice(SpriteNotice::SheetUnreadable {
          path: full_name,
          reason: error.to_string(),
        });

        return Ok(DescriptionSpriteResult::skipped());
      }
    };

    destination.create_root()?;
    destination.create_directory(&relative_path)?;

    let mut written: u32 = 0;
    let mut skipped: u32 = 0;

    for (sprite, sprite_path) in file.sprites.iter().zip(&sprite_paths) {
      let (max_x, max_y) = sprite.get_dimension_boundaries();

      if max_x > dds.width() || max_y > dds.height() {
        if options.is_strict {
          return Err(XrfError::new_texture_processing_error(format!(
            "Unexpected texture '{}' (x:{}, y:{}) boundaries are bigger than source DDS file ({}x{} - {})",
            sprite.id,
            max_x,
            max_y,
            dds.width(),
            dds.height(),
            format_path(&full_name)
          )));
        }

        on_notice(SpriteNotice::SpriteOutsideSheet {
          id: sprite.id.clone(),
          max_x,
          max_y,
          width: dds.width(),
          height: dds.height(),
          sheet: full_name.clone(),
        });

        skipped += 1;

        continue;
      }

      // Unpacked sprites are packing input read at their base level, so a mip chain would only
      // cost space.
      ImageFileWriter::write_dds(
        &destination.prepare_file(sprite_path)?,
        &dds.view(sprite.x, sprite.y, sprite.w, sprite.h).to_image(),
        options.dds_compression_format,
        DdsMipmaps::Disabled,
      )?;

      written += 1;
    }

    Ok(DescriptionSpriteResult::written(written, skipped))
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use image::RgbaImage;
  use xrf_dds::{DdsMipmaps, ImageFormat};
  use xrf_error::XrfError;
  use xrf_test_utils::GeneratedResources;

  use super::DescriptionSpriteUnpacker;
  use crate::report::SpriteNoticeRecorder;
  use crate::{DescriptionSpriteOptions, DescriptionSpriteResult, SpriteNotice, TextureFileDescriptor};
  use xrf_texture::ImageFileWriter;

  fn options_for_missing_source(is_strict: bool) -> DescriptionSpriteOptions {
    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .parent()
      .and_then(|path| path.parent())
      .expect("xrf-sprite to be inside the workspace crates directory")
      .join("target")
      .join("test-resources")
      .join(format!("xrf-sprite-missing-description-{}", std::process::id()));

    DescriptionSpriteOptions {
      listener: None,
      job: Default::default(),
      description: root.join("description.xml"),
      base: root.join("source"),
      output_path: root.join("output"),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files: Vec::new(),
      is_strict,
      link_policy: Default::default(),
      is_forced: false,
    }
  }

  #[test]
  fn skips_an_unreadable_sheet_in_non_strict_mode() {
    let options: DescriptionSpriteOptions = options_for_missing_source(false);
    let file: TextureFileDescriptor = TextureFileDescriptor::new(r"ui\missing");

    assert_eq!(
      DescriptionSpriteUnpacker::unpack_xml_description(&options, &file, &mut |_| {})
        .expect("non-strict unpacking to skip an unreadable sheet"),
      DescriptionSpriteResult::skipped()
    );
    assert!(
      !options
        .output_path
        .join(file.to_host_relative_path().expect("valid logical path"))
        .exists()
    );
  }

  #[test]
  fn rejects_an_unreadable_sheet_in_strict_mode() {
    let options: DescriptionSpriteOptions = options_for_missing_source(true);
    let file: TextureFileDescriptor = TextureFileDescriptor::new(r"ui\missing");

    assert!(DescriptionSpriteUnpacker::unpack_xml_description(&options, &file, &mut |_| {}).is_err());
    assert!(
      !options
        .output_path
        .join(file.to_host_relative_path().expect("valid logical path"))
        .exists()
    );
  }

  const SENTINEL: &[u8] = b"sentinel";

  /// A readable 8x4 sheet at `base/sheet.dds`, a description of `ids` over it, and a sentinel beside `out`.
  fn options_for_sheet(name: &str, ids: &[&str], is_strict: bool) -> DescriptionSpriteOptions {
    let root: PathBuf = GeneratedResources::create_dir(&format!("description-unpack/{name}")).expect("sheet root");

    fs::create_dir_all(root.join("base")).expect("sheet directory");
    fs::write(root.join("victim.dds"), SENTINEL).expect("sentinel beside the destination");
    ImageFileWriter::write_dds(
      &root.join("base").join("sheet.dds"),
      &RgbaImage::new(8, 4),
      ImageFormat::BC3RgbaUnorm,
      DdsMipmaps::Disabled,
    )
    .expect("sheet");

    let textures: String = ids
      .iter()
      .enumerate()
      .map(|(index, id)| {
        format!(
          "<texture id=\"{id}\" x=\"{}\" y=\"0\" width=\"4\" height=\"4\" />",
          (index % 2) * 4
        )
      })
      .collect();

    fs::write(
      root.join("description.xml"),
      format!("<w><file name=\"sheet\">{textures}</file></w>"),
    )
    .expect("description");

    DescriptionSpriteOptions {
      listener: None,
      job: Default::default(),
      description: root.join("description.xml"),
      base: root.join("base"),
      output_path: root.join("out"),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files: Vec::new(),
      is_strict,
      link_policy: Default::default(),
      is_forced: false,
    }
  }

  fn root_of(options: &DescriptionSpriteOptions) -> PathBuf {
    options.base.parent().expect("a test root").to_path_buf()
  }

  #[test]
  fn refuses_a_sprite_id_that_leaves_the_destination() {
    let absolute: String = GeneratedResources::path("description-unpack/absolute/victim")
      .display()
      .to_string();

    for (index, id) in [
      "../../victim",
      r"..\..\victim",
      r"nested\..\..\..\victim",
      "c:victim",
      absolute.as_str(),
    ]
    .into_iter()
    .enumerate()
    {
      for is_strict in [false, true] {
        // A valid sprite is listed first, so a run that wrote before it checked would leave it behind.
        let options: DescriptionSpriteOptions =
          options_for_sheet(&format!("escape-{index}-{is_strict}"), &["ui_valid", id], is_strict);
        let root: PathBuf = root_of(&options);

        assert!(
          matches!(
            DescriptionSpriteUnpacker::unpack_xml_descriptions(&options),
            Err(XrfError::Invalid { .. })
          ),
          "sprite id '{id}' must be refused by name"
        );
        assert_eq!(
          fs::read(root.join("victim.dds")).expect("sentinel"),
          SENTINEL,
          "'{id}' overwrote the sentinel"
        );
        assert!(!root.join("out").exists(), "'{id}' left earlier output behind");
      }
    }
  }

  #[test]
  fn writes_each_sprite_under_its_id_as_spelled() {
    let options: DescriptionSpriteOptions = options_for_sheet("valid", &["ui_icon_PDA", "ui icon (old)"], true);
    let root: PathBuf = root_of(&options);

    let result: DescriptionSpriteResult = DescriptionSpriteUnpacker::unpack_xml_descriptions(&options).expect("unpack");

    assert_eq!((result.files, result.sprites, result.skipped_sprites), (1, 2, 0));
    assert!(root.join("out").join("sheet").join("ui_icon_PDA.dds").is_file());
    assert!(root.join("out").join("sheet").join("ui icon (old).dds").is_file());
    assert_eq!(fs::read(root.join("victim.dds")).expect("sentinel"), SENTINEL);
  }

  #[test]
  fn hands_out_notices_in_selection_order_however_the_workers_finish() {
    let root: PathBuf = GeneratedResources::create_dir("description-unpack/notice-order").expect("description root");
    let names: Vec<String> = (0..16).map(|index| format!("sheet_{index:02}")).collect();
    let files: String = names
      .iter()
      .map(|name| {
        format!(
          "<file name=\"ui\\{name}\"><texture id=\"{name}_icon\" x=\"0\" y=\"0\" width=\"4\" height=\"4\" /></file>"
        )
      })
      .collect();

    fs::write(root.join("description.xml"), format!("<w>{files}</w>")).expect("description");

    // Requested last to first, so neither the description's order nor its map's matches the one asked for. No sheet
    // exists, so each one is passed over with a notice.
    let requested: Vec<String> = names.iter().rev().cloned().collect();
    let recorder: SpriteNoticeRecorder = SpriteNoticeRecorder::default();
    let options: DescriptionSpriteOptions = DescriptionSpriteOptions {
      listener: recorder.new_listener(),
      job: Default::default(),
      description: root.join("description.xml"),
      base: root.join("base"),
      output_path: root.join("out"),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files: requested.clone(),
      is_strict: false,
      link_policy: Default::default(),
      is_forced: false,
    };
    let result: DescriptionSpriteResult = DescriptionSpriteUnpacker::unpack_xml_descriptions(&options)
      .expect("a non-strict run passes unreadable sheets over");
    let notices: Vec<String> = recorder
      .list_notices()
      .into_iter()
      .map(|notice| match notice {
        SpriteNotice::SheetUnreadable { path, .. } => path
          .file_stem()
          .expect("a sheet file name")
          .to_string_lossy()
          .into_owned(),
        other => panic!("expected only unreadable sheets, got {other:?}"),
      })
      .collect();

    assert_eq!(result.skipped_files, 16);
    assert_eq!(notices, requested);
  }

  #[test]
  fn refuses_a_sprite_already_there_in_any_case_unless_forced() {
    let options: DescriptionSpriteOptions = options_for_sheet("existing", &["ui_icon", "ui_other"], true);
    let existing: PathBuf = root_of(&options).join("out").join("sheet").join("UI_Icon.dds");

    fs::create_dir_all(existing.parent().expect("a sheet folder")).expect("sheet folder");
    fs::write(&existing, SENTINEL).expect("an earlier sprite");

    assert!(
      DescriptionSpriteUnpacker::unpack_xml_descriptions(&options)
        .expect_err("an earlier sprite is refused")
        .to_string()
        .contains("UI_Icon.dds")
    );
    assert_eq!(fs::read(&existing).expect("the earlier sprite"), SENTINEL);
    assert!(
      !root_of(&options)
        .join("out")
        .join("sheet")
        .join("ui_other.dds")
        .exists()
    );

    let forced: DescriptionSpriteOptions = DescriptionSpriteOptions {
      listener: None,
      is_forced: true,
      ..options
    };

    assert_eq!(
      DescriptionSpriteUnpacker::unpack_xml_descriptions(&forced)
        .expect("forced, it writes over")
        .sprites,
      2
    );
  }
}
