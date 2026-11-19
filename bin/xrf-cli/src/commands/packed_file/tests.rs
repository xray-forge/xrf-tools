use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_error::XrfResult;
use xrf_particles::ParticlesFile;
use xrf_test_utils::GeneratedResources;

use crate::commands::packed_file::{PackCommand, UnpackCommand, VerifyCommand};
use crate::commands::particle::fixtures::create_library;
use crate::core::command::command_testing::{run_command, run_command_for_result};
use crate::core::command::{CommandResult, GenericCommand};

fn run<T: GenericCommand>(command: &T, path: &Path, extra: &[&str]) -> CommandResult {
  let mut arguments: Vec<String> = vec![
    String::from(command.operation()),
    path.display().to_string(),
    String::from("--silent"),
  ];

  arguments.extend(extra.iter().map(|it| String::from(*it)));

  run_command(command, &arguments)
}

#[test]
fn packing_requires_a_destination() -> XrfResult {
  let library: PathBuf = create_library("packed_file_dest_required")?;

  assert!(
    run(
      &PackCommand::<ParticlesFile>::default(),
      &library.with_file_name("unpacked"),
      &[]
    )
    .is_err()
  );

  Ok(())
}

#[test]
fn packing_refuses_a_destination_that_is_there_and_keeps_it() -> XrfResult {
  let library: PathBuf = create_library("packed_file_refused")?;
  let unpacked: PathBuf = library.with_file_name("unpacked");

  fs::write(&library, b"previous")?;

  let refused = run(
    &PackCommand::<ParticlesFile>::default(),
    &unpacked,
    &["--dest", &library.display().to_string()],
  );

  assert!(refused.is_err());
  assert_eq!(fs::read(&library)?, b"previous");

  run(
    &PackCommand::<ParticlesFile>::default(),
    &unpacked,
    &["--dest", &library.display().to_string(), "--force"],
  )
  .expect("a forced pack replaces it");

  assert!(ParticlesFile::read_from_path::<xrf_spawn::XRayByteOrder, _>(&library).is_ok());

  Ok(())
}

/// The old folder stands until the new one is complete, so a source that cannot be read costs nothing.
#[test]
fn a_forced_unpack_of_a_bad_source_keeps_the_old_folder() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("packed_file/unpack_bad_source")?;
  let source: PathBuf = root.join("broken.xr");
  let dest: PathBuf = root.join("unpacked");

  fs::write(&source, b"not a particles file")?;
  fs::create_dir_all(&dest)?;
  fs::write(dest.join("kept.ltx"), b"kept")?;

  assert!(
    run(
      &UnpackCommand::<ParticlesFile>::default(),
      &source,
      &["--dest", &dest.display().to_string(), "--force"],
    )
    .is_err()
  );
  assert_eq!(fs::read(dest.join("kept.ltx"))?, b"kept");

  Ok(())
}

#[test]
fn a_forced_unpack_replaces_the_old_folder_whole() -> XrfResult {
  let library: PathBuf = create_library("packed_file_unpack_replaces")?;
  let dest: PathBuf = library.with_file_name("replaced");

  fs::create_dir_all(&dest)?;
  fs::write(dest.join("stale.ltx"), b"stale")?;

  run(
    &UnpackCommand::<ParticlesFile>::default(),
    &library,
    &["--dest", &dest.display().to_string(), "--force"],
  )
  .expect("the unpack");

  assert!(!dest.join("stale.ltx").exists());
  assert!(dest.join("effects.ltx").exists());

  Ok(())
}

#[test]
fn reports_an_unpack_of_a_format_saying_nothing_more_as_the_conversion() -> CommandResult {
  let library: PathBuf = create_library("packed_file_unpack_report")?;
  let dest: PathBuf = library.with_file_name("reported");
  let command: UnpackCommand<ParticlesFile> = UnpackCommand::default();
  let report: Value = run_command_for_result(
    &command,
    &[
      String::from(command.operation()),
      library.display().to_string(),
      String::from("--dest"),
      dest.display().to_string(),
      String::from("--force"),
      String::from("--silent"),
      String::from("--json"),
    ],
  )?
  .expect("a result was asked for");
  let mut keys: Vec<&String> = report.as_object().expect("an object").keys().collect();

  keys.sort();

  assert_eq!(keys, ["destination", "readDuration", "source", "writeDuration"]);

  Ok(())
}

#[test]
fn verifies_a_library_packed_or_unpacked_and_judges_a_broken_one() -> XrfResult {
  let library: PathBuf = create_library("packed_file_verify")?;
  let broken: PathBuf = library.with_file_name("broken.xr");

  fs::write(&broken, b"not a particles file")?;

  assert!(run(&VerifyCommand::<ParticlesFile>::default(), &library, &[]).is_ok());
  assert!(
    run(
      &VerifyCommand::<ParticlesFile>::default(),
      &library.with_file_name("unpacked"),
      &["--unpacked"],
    )
    .is_ok()
  );
  assert_eq!(
    run(&VerifyCommand::<ParticlesFile>::default(), &broken, &[])
      .expect_err("a broken file is judged")
      .exit_code(),
    3
  );

  Ok(())
}
