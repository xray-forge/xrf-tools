//! What `object compile` refuses, that a refusal writes nothing, and which tangent method it compiles with.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_chunk::{ChunkLayout, XRayByteOrder};
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_rawdata::{ObjectProvenance, TangentMethod};
use xrf_test_utils::GeneratedResources;

use crate::commands::object::compile::command::CompileCommand;
use crate::core::command::CommandResult;
use crate::core::command::command_testing::run_command;

fn compile(source: &Path, destination: &Path, extra: &[&str]) -> CommandResult {
  let mut arguments: Vec<String> = vec![
    String::from("compile"),
    String::from("--path"),
    source.display().to_string(),
    String::from("--dest"),
    destination.display().to_string(),
    String::from("--silent"),
  ];

  arguments.extend(extra.iter().map(|it| String::from(*it)));

  run_command(&CompileCommand, &arguments)
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

fn get_error(verdict: CommandResult) -> String {
  match verdict {
    Ok(()) => panic!("Expect the compile to be refused"),
    Err(error) => error.to_string(),
  }
}

#[test]
fn refuses_to_replace_an_ogf_without_force() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("object-compile/existing")?;
  let source: PathBuf = root.join("model.object");
  let destination: PathBuf = root.join("model.ogf");

  empty_object(EditableObjectFile::DYNAMIC_FLAG).write_to_path::<XRayByteOrder, _>(&source)?;
  fs::write(&destination, b"exported")?;

  let error: String = get_error(compile(&source, &destination, &[]));

  assert!(
    error.contains("--force"),
    "Expect the refusal to name the flag: {error}"
  );
  assert_eq!(fs::read(&destination)?, b"exported");

  Ok(())
}

#[test]
fn takes_the_tangent_method_a_sidecar_records_unless_told_otherwise() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("object-compile/tangents")?;
  let (decompiled, authored) = (root.join("decompiled.object"), root.join("authored.object"));
  let provenance: ObjectProvenance = ObjectProvenance::new(
    "meshes/decompiled.ogf",
    b"ogf",
    "xrf-cli object decompile",
    "1.0.0",
    Vec::new(),
    TangentMethod::MikkTSpace,
  );

  fs::write(ObjectProvenance::sidecar_path(&decompiled), provenance.to_json()?)?;

  let tangents = |source: &Path, choice: &str| CompileCommand::get_options(source, choice).map(|it| it.tangents);

  assert_eq!(tangents(&decompiled, "auto")?, TangentMethod::MikkTSpace);
  assert_eq!(tangents(&decompiled, "actor-editor")?, TangentMethod::ActorEditor);
  assert_eq!(tangents(&authored, "auto")?, TangentMethod::ActorEditor);
  assert_eq!(tangents(&authored, "mikktspace")?, TangentMethod::MikkTSpace);

  Ok(())
}

#[test]
fn refuses_a_static_object_and_writes_nothing() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("object-compile/static")?;
  let source: PathBuf = root.join("model.object");
  let destination: PathBuf = root.join("model.ogf");

  empty_object(0).write_to_path::<XRayByteOrder, _>(&source)?;

  let error: String = get_error(compile(&source, &destination, &["--force"]));

  assert!(
    error.contains("static object"),
    "Expect the refusal to name the static object: {error}"
  );
  assert!(!destination.exists());

  Ok(())
}
