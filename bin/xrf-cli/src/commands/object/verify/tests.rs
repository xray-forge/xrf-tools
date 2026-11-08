//! What `object verify` refuses, and how its differences become checks; what counts as a difference is `xrf-rawdata`'s
//! to pin.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use xrf_chunk::{ChunkLayout, ChunkWriter, XRayByteOrder};
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_ogf::OgfHeaderChunk;
use xrf_rawdata::ObjectVerification;
use xrf_report::{Report, Status};
use xrf_test_utils::GeneratedResources;

use crate::commands::object::verify::command::VerifyCommand;
use crate::core::command::CommandResult;
use crate::core::command::command_testing::run_command;

fn verify(source: &Path, reference: &Path) -> CommandResult {
  run_command(
    &VerifyCommand,
    &[
      String::from("verify"),
      String::from("--path"),
      source.display().to_string(),
      String::from("--ogf"),
      reference.display().to_string(),
      String::from("--silent"),
    ],
  )
}

/// An object of no geometry, dynamic where `flags` says so.
fn empty_object(flags: u32) -> EditableObjectFile {
  EditableObjectFile {
    flags,
    flags_size: EditableObjectFile::FLAGS_SIZE,
    class_script: None,
    lods: None,
    meshes: None,
    surfaces: None,
    bones: None,
    embedded_motions: None,
    motion_references: None,
    bone_parts: None,
    actor_transform: None,
    description: None,
    layout: ChunkLayout::default(),
  }
}

/// An OGF holding only the header of a rigid skeleton.
fn header_only() -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.write_xr::<XRayByteOrder, _>(&OgfHeaderChunk {
    version: 4,
    model_type: 10,
    shader_id: 0,
    bounding_box: Aabb {
      min: Vector3d::ZERO,
      max: Vector3d::new(1.0, 1.0, 1.0),
    },
    bounding_sphere: Sphere {
      center: Vector3d::ZERO,
      radius: 1.0,
    },
  })?;

  writer.flush_chunk_into_buffer::<XRayByteOrder>(OgfHeaderChunk::CHUNK_ID)
}

/// `object` and a header-only reference, written to a fresh directory named `name`.
fn write_pair(name: &str, object: &EditableObjectFile) -> XrfResult<(PathBuf, PathBuf)> {
  let root: PathBuf = GeneratedResources::create_dir(&format!("object-verify/{name}"))?;
  let (source, reference) = (root.join("model.object"), root.join("model.ogf"));

  object.write_to_path::<XRayByteOrder, _>(&source)?;
  fs::write(&reference, header_only()?)?;

  Ok((source, reference))
}

fn get_error(verdict: CommandResult) -> String {
  match verdict {
    Ok(()) => panic!("Expect the verification to be refused"),
    Err(error) => error.to_string(),
  }
}

#[test]
fn refuses_an_object_the_compile_refuses() -> XrfResult {
  let (source, reference) = write_pair("empty", &empty_object(0))?;
  let error: String = get_error(verify(&source, &reference));

  assert!(
    error.contains("no mesh"),
    "Expect the refusal to name the missing mesh: {error}"
  );

  Ok(())
}

#[test]
fn refuses_a_reference_that_is_not_an_ogf() -> XrfResult {
  let (source, reference) = write_pair("unread", &empty_object(EditableObjectFile::DYNAMIC_FLAG))?;

  fs::write(&reference, b"not an ogf")?;

  assert!(verify(&source, &reference).is_err());

  Ok(())
}

#[test]
fn checks_each_area_and_fails_only_where_it_differs() {
  let report: Report = VerifyCommand::report(
    &ObjectVerification {
      skeleton: vec![String::from("bone 2: tip under 'b', the reference's c under 'b'")],
      ..ObjectVerification::default()
    },
    Duration::ZERO,
  );
  let statuses: Vec<(String, Status, usize)> = report
    .checks()
    .iter()
    .map(|check| (check.id().to_string(), check.status(), check.findings().len()))
    .collect();

  assert_eq!(
    statuses,
    [
      (String::from("object.model"), Status::Passed, 0),
      (String::from("object.skeleton"), Status::Failed, 1),
      (String::from("object.motions"), Status::Passed, 0),
      (String::from("object.geometry"), Status::Passed, 0),
    ]
  );
  assert_eq!(report.status(), Status::Failed);
}
