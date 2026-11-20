//! What `thm write` gives a texture that has no descriptor, and when it refuses to.

use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_chunk::XRayByteOrder;
use xrf_dds::{DdsEncoding, DdsMipChain, DdsMipFilter, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage};
use xrf_test_utils::GeneratedResources;
use xrf_thm::{ThmFile, ThmFormat, ThmTextureFlag, ThmTextureParamChunk};

use crate::commands::thm::write::command::WriteCommand;
use crate::core::command::CommandResult;
use crate::core::command::command_testing::run_command_for_result;

fn texture(case: &str) -> CommandResult<PathBuf> {
  let path: PathBuf = GeneratedResources::path(&format!("thm_write/{case}/wall.dds"));

  std::fs::create_dir_all(path.parent().expect("case directory"))?;

  let _ = std::fs::remove_file(path.with_extension("thm"));

  DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Fast)
    .encode(&DdsMipChain::build(
      &RgbaImage::from_pixel(32, 16, Rgba([90, 80, 70, 200])),
      DdsMipmaps::Filtered(DdsMipFilter::Box),
    )?)?
    .write_to_path(&path)?;

  Ok(path)
}

fn run(source: &Path, arguments: &[&str]) -> CommandResult<Value> {
  let mut command: Vec<String> = vec![String::from("write"), source.display().to_string()];

  command.extend(arguments.iter().map(|it| (*it).to_owned()));
  command.push(String::from("--json"));

  Ok(run_command_for_result(&WriteCommand, &command)?.expect("Expect a reported result"))
}

fn read(path: &Path) -> CommandResult<ThmFile> {
  Ok(ThmFile::read_from_path::<XRayByteOrder, _>(&path)?)
}

#[test]
fn gives_a_texture_without_one_the_sdk_defaults_with_its_size_and_format() -> CommandResult {
  let source: PathBuf = texture("defaults")?;
  let report: Value = run(&source, &[])?;
  let written: ThmFile = read(&source.with_extension("thm"))?;
  let mut expected: ThmFile = ThmFile::new_texture();
  let param: &mut ThmTextureParamChunk = expected.texture_param.as_mut().expect("parameters");

  param.format = ThmFormat::Dxt5;
  param.width = 32;
  param.height = 16;
  param.flags.set(ThmTextureFlag::HasAlpha, true);

  assert_eq!(written, expected);
  assert_eq!(report["isDryRun"], false);
  assert!(
    report["destination"]
      .as_str()
      .is_some_and(|it| it.ends_with("wall.thm"))
  );
  assert!(report["source"].as_str().is_some_and(|it| it.ends_with("wall.dds")));
  assert_eq!(report["descriptor"]["textureParam"]["width"], 32);

  Ok(())
}

#[test]
fn sets_the_fields_named_over_the_defaults() -> CommandResult {
  let source: PathBuf = texture("fields")?;

  run(
    &source,
    &[
      "--detail-name",
      "detail\\detail_grnd_earth",
      "--set-flag",
      "flDiffuseDetail",
      "--clear-flag",
      "flDitherColor",
      "--fade-color",
      "7F7F7F7F",
    ],
  )?;

  let written: ThmFile = read(&source.with_extension("thm"))?;
  let param: ThmTextureParamChunk = written.texture_param.expect("parameters");

  assert_eq!(
    written.detail.map(|detail| detail.name),
    Some(String::from("detail\\detail_grnd_earth"))
  );
  assert!(param.flags.has(ThmTextureFlag::DiffuseDetail));
  assert!(!param.flags.has(ThmTextureFlag::DitherColor));
  assert_eq!(param.fade_color, 0x7F7F_7F7F);

  Ok(())
}

#[test]
fn refuses_a_descriptor_already_there_unless_forced() -> CommandResult {
  let source: PathBuf = texture("existing")?;

  run(&source, &[])?;

  assert!(run(&source, &["--detail-name", "other"]).is_err());
  assert_eq!(
    read(&source.with_extension("thm"))?.detail.map(|detail| detail.name),
    Some(String::new())
  );

  run(&source, &["--detail-name", "other", "-f"])?;

  assert_eq!(
    read(&source.with_extension("thm"))?.detail.map(|detail| detail.name),
    Some(String::from("other"))
  );

  Ok(())
}

#[test]
fn writes_where_told_and_nothing_on_a_dry_run() -> CommandResult {
  let source: PathBuf = texture("destination")?;
  let destination: PathBuf = source.with_file_name("named.thm");
  let _ = std::fs::remove_file(&destination);

  let report: Value = run(&source, &["-d", &destination.display().to_string(), "--dry-run"])?;

  assert_eq!(report["isDryRun"], true);
  assert!(!destination.exists());

  run(&source, &["-d", &destination.display().to_string()])?;

  assert!(destination.is_file());
  assert!(!source.with_extension("thm").exists());

  Ok(())
}

#[test]
fn refuses_a_flag_both_set_and_cleared() -> CommandResult {
  let source: PathBuf = texture("contradiction")?;

  assert!(run(&source, &["--set-flag", "flHasAlpha", "--clear-flag", "flHasAlpha"]).is_err());
  assert!(!source.with_extension("thm").exists());

  Ok(())
}
