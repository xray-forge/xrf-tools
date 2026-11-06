//! What `object decompile` refuses, and that a refusal writes nothing.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_ogf::OgfHeaderChunk;
use xrf_test_utils::GeneratedResources;

use crate::commands::object::decompile::command::DecompileCommand;
use crate::core::command::CommandResult;
use crate::core::command::command_testing::run_command;

fn decompile(source: &Path, destination: &Path, extra: &[&str]) -> CommandResult {
  let mut arguments: Vec<String> = vec![
    String::from("decompile"),
    String::from("--path"),
    source.display().to_string(),
    String::from("--dest"),
    destination.display().to_string(),
    String::from("--silent"),
  ];

  arguments.extend(extra.iter().map(|it| String::from(*it)));

  run_command(&DecompileCommand, &arguments)
}

/// `MT_NORMAL` and `MT_SKELETON_RIGID` as a header stores them.
const NORMAL: u8 = 0;
const SKELETON_RIGID: u8 = 10;

/// An OGF holding only a header of `model_type`.
fn header_only(model_type: u8) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.write_xr::<XRayByteOrder, _>(&OgfHeaderChunk {
    version: 4,
    model_type,
    shader_id: 0,
    bounding_box: Aabb {
      min: Vector3d::new(0.0, 0.0, 0.0),
      max: Vector3d::new(1.0, 1.0, 1.0),
    },
    bounding_sphere: Sphere {
      center: Vector3d::new(0.0, 0.0, 0.0),
      radius: 1.0,
    },
  })?;

  writer.flush_chunk_into_buffer::<XRayByteOrder>(OgfHeaderChunk::CHUNK_ID)
}

fn get_error(verdict: CommandResult) -> String {
  match verdict {
    Ok(()) => panic!("Expect the decompile to be refused"),
    Err(error) => error.to_string(),
  }
}

#[test]
fn refuses_to_replace_an_object_without_force() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("object-decompile/existing")?;
  let source: PathBuf = root.join("model.ogf");
  let destination: PathBuf = root.join("model.object");

  fs::write(&source, header_only(SKELETON_RIGID)?)?;
  fs::write(&destination, b"authored")?;

  let error: String = get_error(decompile(&source, &destination, &[]));

  assert!(
    error.contains("--force"),
    "Expect the refusal to name the flag: {error}"
  );
  assert_eq!(fs::read(&destination)?, b"authored");
  assert!(!root.join("model.object.xrf.json").exists());

  Ok(())
}

#[test]
fn refuses_a_model_that_is_not_a_skeleton_and_writes_nothing() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("object-decompile/static")?;
  let source: PathBuf = root.join("model.ogf");
  let destination: PathBuf = root.join("model.object");

  fs::write(&source, header_only(NORMAL)?)?;

  let error: String = get_error(decompile(&source, &destination, &["--force"]));

  assert!(
    error.contains("MT_NORMAL"),
    "Expect the refusal to name the model type: {error}"
  );
  assert!(!destination.exists());
  assert!(!root.join("model.object.xrf.json").exists());

  Ok(())
}
