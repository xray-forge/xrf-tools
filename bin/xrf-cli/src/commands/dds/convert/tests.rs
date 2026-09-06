//! What `dds convert` writes, and what its report says the choice cost.

use std::path::PathBuf;

use serde_json::Value;
use xrf_dds::{DdsEncoding, DdsFile, DdsMipChain, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::commands::dds::convert::command::ConvertCommand;
use crate::core::command_testing::run_command_for_result;
use crate::core::generic_command::CommandResult;

/// A source texture with detail in every channel, so a lossy candidate has something to lose.
fn source(case: &str) -> CommandResult<PathBuf> {
  let path: PathBuf = build_absolute_generated_test_resource_path(&format!("dds_convert/{case}/source.dds"));

  std::fs::create_dir_all(path.parent().expect("case directory"))?;

  let base: RgbaImage = RgbaImage::from_fn(32, 32, |x, y| {
    Rgba([(x * 7 + y * 13) as u8, (x * 31 + y * 3) as u8, (x * 17) as u8, u8::MAX])
  });

  DdsEncoding::new(ImageFormat::Rgba8Unorm, Quality::Fast)
    .encode(&DdsMipChain::build(&base, DdsMipmaps::Disabled)?)?
    .write_to_path(&path)?;

  Ok(path)
}

fn run(case: &str, arguments: &[&str]) -> CommandResult<(PathBuf, Value)> {
  let source: PathBuf = source(case)?;
  let destination: PathBuf = source.with_file_name("out.dds");

  let mut command: Vec<String> = vec![
    String::from("convert"),
    source.display().to_string(),
    destination.display().to_string(),
  ];

  command.extend(arguments.iter().map(|it| (*it).to_owned()));
  command.push(String::from("--json"));

  let result: Value = run_command_for_result(&ConvertCommand, &command)?.expect("Expect a reported result");

  Ok((destination, result))
}

#[test]
fn writes_the_requested_format_and_prices_it() -> CommandResult {
  let (destination, report) = run("written", &["--format", "bc3"])?;

  assert_eq!(report["written"]["format"], "BC3 (DXT5)");
  assert_eq!(report["mipmapLevels"], 6, "expect a chain unless one is refused");
  assert_eq!(report["width"], 32);

  // A lossy format costs something measurable, and the report says how much.
  assert!(report["written"]["psnr"].as_f64().is_some_and(|psnr| psnr > 0.0));
  assert!(report["written"]["gpuBytes"].as_u64().is_some_and(|bytes| bytes > 0));

  // Nothing else was weighed, because nothing asked.
  assert_eq!(report["candidates"].as_array().map(Vec::len), Some(0));

  assert_eq!(
    DdsFile::read_from_path(&destination)?.metadata().mipmap_levels,
    6,
    "expect the file on disk to carry what the report claims"
  );

  Ok(())
}

#[test]
fn writing_without_a_chain_leaves_only_the_base() -> CommandResult {
  let (_, report) = run("flat", &["--format", "bc1", "--no-mipmaps"])?;

  assert_eq!(report["mipmapLevels"], 1);

  Ok(())
}

#[test]
fn comparing_weighs_every_other_candidate_against_the_same_levels() -> CommandResult {
  // Four more encodes, which is why it is a flag: the point is to choose, so every candidate has to be measured
  // against the same reduced levels rather than against a chain of its own.
  let (_, report) = run("compared", &["--format", "bc3", "--compare"])?;
  let candidates = report["candidates"].as_array().expect("Expect the other candidates");

  assert_eq!(candidates.len(), 4);
  assert!(
    candidates.iter().all(|it| it["format"] != "BC3 (DXT5)"),
    "expect the written format to be reported once, not twice"
  );

  // Uncompressed is the one candidate that loses nothing, and it is the largest.
  let uncompressed = candidates
    .iter()
    .find(|it| it["format"] == "RGBA8")
    .expect("Expect RGBA8 among the candidates");

  assert!(
    uncompressed["psnr"].is_null(),
    "expect no loss where nothing is compressed"
  );
  assert!(uncompressed["gpuBytes"].as_u64() > report["written"]["gpuBytes"].as_u64());

  Ok(())
}

#[test]
fn every_candidate_says_which_renderer_paths_load_it() -> CommandResult {
  let (_, report) = run("compatibility", &["--format", "bc7"])?;
  let compatibility = report["written"]["compatibility"]
    .as_array()
    .expect("Expect a renderer table");

  // BC7 is the one candidate that costs a renderer, and GL was never read either way.
  assert_eq!(compatibility.len(), 3);
  assert_eq!(compatibility[0]["renderer"], "DX9");
  assert_eq!(compatibility[0]["support"], "unsupported");
  assert_eq!(compatibility[1]["support"], "supported");
  assert_eq!(compatibility[2]["support"], "unverified");

  Ok(())
}

#[test]
fn refuses_a_format_it_has_no_encoder_for() -> CommandResult {
  let source: PathBuf = source("refused")?;

  assert!(
    run_command_for_result(
      &ConvertCommand,
      &[
        String::from("convert"),
        source.display().to_string(),
        source.with_file_name("out.dds").display().to_string(),
        String::from("--format"),
        String::from("bc4"),
      ],
    )
    .is_err(),
    "expect a format outside the candidate list to be a usage error"
  );

  Ok(())
}
