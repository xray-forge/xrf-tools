//! What `dds make-bump` generates from a height map, and what it says about the gloss.

use std::path::PathBuf;

use serde_json::Value;
use xrf_dds::{DdsFile, Rgba, RgbaImage};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::commands::dds::make_bump::command::MakeBumpCommand;
use crate::core::command_testing::run_command_for_result;
use crate::core::generic_command::CommandResult;

/// A height map whose relief climbs with y, written as a png the way an author would hand one over.
fn height(case: &str) -> CommandResult<PathBuf> {
  let path: PathBuf = build_absolute_generated_test_resource_path(&format!("dds_make_bump/{case}/height.png"));

  std::fs::create_dir_all(path.parent().expect("case directory"))?;

  RgbaImage::from_fn(16, 16, |_, y| {
    let level: u8 = (y * 16) as u8;

    Rgba([level, level, level, u8::MAX])
  })
  .save(&path)
  .expect("Expect the height map to be written");

  Ok(path)
}

fn run(case: &str, arguments: &[&str]) -> CommandResult<(PathBuf, Value)> {
  let source: PathBuf = height(case)?;
  let destination: PathBuf = source.with_file_name("act_wall");

  let mut command: Vec<String> = vec![
    String::from("make-bump"),
    source.display().to_string(),
    destination.display().to_string(),
  ];

  command.extend(arguments.iter().map(|it| (*it).to_owned()));
  command.push(String::from("--json"));

  let result: Value = run_command_for_result(&MakeBumpCommand, &command)?.expect("Expect a reported result");

  Ok((destination, result))
}

#[test]
fn writes_both_halves_beside_the_texture_they_belong_to() -> CommandResult {
  let (destination, report) = run("pair", &[])?;

  let bump: PathBuf = destination.with_file_name("act_wall_bump.dds");
  let companion: PathBuf = destination.with_file_name("act_wall_bump#.dds");

  assert!(bump.is_file() && companion.is_file());
  assert!(
    report["bump"]
      .as_str()
      .is_some_and(|it| it.ends_with("act_wall_bump.dds"))
  );
  assert!(
    report["companion"]
      .as_str()
      .is_some_and(|it| it.ends_with("act_wall_bump%23.dds") || it.ends_with("act_wall_bump#.dds"))
  );

  // Both halves are DXT5 with a chain, as the SDK writes them.
  assert_eq!(DdsFile::read_from_path(&bump)?.metadata().mipmap_levels, 5);
  assert_eq!(DdsFile::read_from_path(&companion)?.metadata().mipmap_levels, 5);

  Ok(())
}

#[test]
fn reports_the_gloss_it_was_given() -> CommandResult {
  let (_, report) = run("gloss", &["--gloss-constant", "0.75"])?;

  assert!(
    report["glossPower"].as_f64().is_some_and(|it| (it - 0.75).abs() < 0.01),
    "got {}",
    report["glossPower"]
  );
  assert_eq!(report["isGlossTooDark"], false);

  Ok(())
}

#[test]
fn a_gloss_too_dark_is_a_verdict_and_not_a_failure() -> CommandResult {
  // The SDK answers -1000 and keeps the files it wrote, because a matte surface is a thing somebody may have meant.
  let (destination, report) = run("dark", &["--gloss-constant", "0.05"])?;

  assert_eq!(report["isGlossTooDark"], true);
  assert!(destination.with_file_name("act_wall_bump.dds").is_file());

  Ok(())
}

#[test]
fn refuses_a_gloss_outside_the_range_a_gloss_is_measured_in() -> CommandResult {
  assert!(run("invalid", &["--gloss-constant", "4"]).is_err());

  Ok(())
}
