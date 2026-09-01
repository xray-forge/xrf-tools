//! Exercises how `archive pack` reads a volume-size request, which is the one place the engine's cap can be lifted.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_pack::{VOLUME_SIZE_HARD_MAX, VOLUME_SIZE_MAX};

use crate::commands::archive::pack::command::PackCommand;
use crate::core::command_testing::run_command;
use crate::core::generic_command::CommandResult;

/// A source tree holding one small file, and a destination beside it that packing creates.
fn create_roots(name: &str) -> XrfResult<(PathBuf, PathBuf)> {
  let root: PathBuf = std::env::temp_dir().join(format!("xrf-cli-pack-{name}-{}", std::process::id()));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(root.join("configs"))?;
  fs::write(
    root.join("configs").join("system.ltx"),
    "[section]
value = 1
",
  )?;

  Ok((root.clone(), root.join("packed")))
}

fn pack(source: &Path, destination: &Path, megabytes: u64, is_oversized_allowed: bool) -> CommandResult {
  let mut arguments: Vec<String> = vec![
    String::from("pack"),
    String::from("--path"),
    source.display().to_string(),
    String::from("--dest"),
    destination.display().to_string(),
    String::from("--max-size"),
    megabytes.to_string(),
    String::from("--silent"),
  ];

  if is_oversized_allowed {
    arguments.push(String::from("--oversized-volumes"));
  }

  run_command(&PackCommand, &arguments)
}

/// Megabytes just past a byte bound, which is the unit the flag is entered in.
fn megabytes_past(bytes: u64) -> u64 {
  bytes / xrf_utils::BYTES_PER_MEGABYTE + 1
}

#[test]
fn refuses_a_cap_past_what_the_engine_mounts() -> CommandResult {
  let (source, destination) = create_roots("oversized-refused")?;
  let error: String = pack(&source, &destination, megabytes_past(VOLUME_SIZE_MAX), false)
    .expect_err("the engine cap stands unless it is lifted")
    .to_string();

  assert!(error.contains("--oversized-volumes"), "{error}");
  assert!(!destination.exists(), "a refused request writes nothing");

  fs::remove_dir_all(source)?;

  Ok(())
}

/// The flag is read before the size, so lifting the cap and naming the size in one invocation works whatever order
/// clap reports them in.
#[test]
fn packs_past_that_cap_when_the_flag_lifts_it() -> CommandResult {
  let (source, destination) = create_roots("oversized-allowed")?;

  pack(&source, &destination, megabytes_past(VOLUME_SIZE_MAX), true)?;

  assert!(destination.join("gamedata.db").is_file(), "the set is published");

  fs::remove_dir_all(source)?;

  Ok(())
}

/// The typo guard is not what the flag lifts: a stray digit is refused with the flag on.
#[test]
fn refuses_a_mistyped_cap_even_with_the_flag() -> CommandResult {
  let (source, destination) = create_roots("mistyped-refused")?;
  let error: String = pack(&source, &destination, megabytes_past(VOLUME_SIZE_HARD_MAX), true)
    .expect_err("nothing lifts the hard bound")
    .to_string();

  assert!(error.contains("stray digit"), "{error}");
  assert!(!destination.exists(), "a refused request writes nothing");

  fs::remove_dir_all(source)?;

  Ok(())
}
