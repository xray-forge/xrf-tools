use std::collections::HashMap;

use serde::Serialize;
use xrf_db::ShaderBlender;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;
use crate::plugins::archives::describe::shaders::archive_shaders_property::ArchiveShadersProperty;

/// One definition of the library: what a shader name resolves to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveShadersBlender {
  /// The shader name a mesh, a level surface or a config declares to reach this definition.
  pub name: String,
  /// The class tag, which decides which passes are compiled and how its properties are read.
  pub class: String,
  /// The class's own format version, which decides the order its `Load` reads properties in.
  pub version: u16,
  /// Machine the SDK last saved it on, the file's only provenance; empty where it carries none.
  pub computer: String,
  /// Save time as the SDK stored it, which is a `u32` of its own and not an instant this can date.
  pub time: u32,
  pub properties: Vec<ArchiveShadersProperty>,
}

impl ArchiveShadersBlender {
  /// Every blender of a library, in name order.
  pub fn of_all<'a>(source: &ArchiveDescribeSource, blenders: impl Iterator<Item = &'a ShaderBlender>) -> Vec<Self> {
    // Shared across the library rather than per blender: 713 of vanilla's 205 blenders' properties are textures and
    // they name far fewer distinct files, while a lookup that misses walks the whole name table.
    let mut resolved: HashMap<String, ArchiveReference> = HashMap::new();

    let mut described: Vec<Self> = blenders
      .map(|blender| Self {
        name: blender.name.clone(),
        class: blender.class.tag(),
        version: blender.version,
        computer: blender.computer.clone(),
        time: blender.time,
        properties: ArchiveShadersProperty::of_all(source, &blender.properties, &mut resolved),
      })
      .collect();

    described.sort_by(|left, right| left.name.cmp(&right.name));
    described
  }
}
