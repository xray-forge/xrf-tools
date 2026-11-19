use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_ogf::{OgfFile, OgfGeometry, OgfModelType, OgfStaticVertex, OgfVertexData};
use xrf_rawdata::{ObjectCompileOptions, ObjectCompileSource, ObjectCompiler};
use xrf_visual::{
  VisualMesh, VisualMotions, VisualPackage, VisualPacker, convert_declared_bounds, convert_uvs, convert_vector,
};

/// An SDK editable object as the renderer draws it: compiled as the Actor Editor's `Export OGF` writes it, then packed
/// as a model is, so it draws exactly as the visual the game would load from it.
pub struct ObjectVisual {
  /// The model packed for the renderer, in renderer space: a rigid object in its own space, a skeleton in its bind
  /// pose with its skin.
  pub package: VisualPackage,
  /// What posing needs of a skeleton object, or `None` for one carrying no bind pose.
  pub motions: Option<VisualMotions>,
}

impl ObjectVisual {
  /// `object` compiled as `xrf-cli object compile` writes it, then read back as the engine reads the file, so nothing
  /// the file would not keep reaches the renderer. `options` covers the choices the editor's export leaves open.
  ///
  /// # Errors
  ///
  /// Returns an error where the object does not compile, or its compiled file does not read back.
  pub fn compile(
    object: &EditableObjectFile,
    source: &ObjectCompileSource,
    options: &ObjectCompileOptions,
  ) -> XrfResult<Self> {
    let file: OgfFile = ObjectCompiler::compile_with(object, source, options)
      .and_then(|compiled| OgfFile::read_from_bytes::<XRayByteOrder>(compiled.bytes))?;

    Ok(Self {
      package: Self::pack(&file),
      motions: VisualMotions::of(&file),
    })
  }

  /// The compiled file packed for the renderer. `VisualPacker::pack` packs skinned vertices alone, so a rigid object's
  /// static parts are packed here as meshes, tangents derived from their uvs, under the description its file declares.
  fn pack(file: &OgfFile) -> VisualPackage {
    let Some(meshes) = Self::list_static_meshes(file) else {
      return VisualPacker::pack(file);
    };
    let mut package: VisualPackage = VisualPacker::pack_meshes(&meshes);

    package.description.version = file.header.version;
    package.description.model_type = file.header.model_type;
    package.description.model_type_label = OgfModelType::label(file.header.model_type);
    package.description.shader_id = file.header.shader_id;
    package.description.source_file = file
      .get_description()
      .map(|description| description.source_file.clone())
      .filter(|source_file| !source_file.is_empty());
    package.description.declared_bounds =
      convert_declared_bounds(&file.header.bounding_box, &file.header.bounding_sphere);

    package
  }

  /// Each part as a renderer-space mesh in file order, or `None` where any part holds no static vertices.
  fn list_static_meshes(file: &OgfFile) -> Option<Vec<VisualMesh>> {
    let parts: Vec<&OgfFile> = match file.children.as_ref().map(|children| children.nested.as_slice()) {
      Some(nested) if !nested.is_empty() => nested.iter().collect(),
      _ => vec![file],
    };

    parts.into_iter().map(Self::read_static_mesh).collect()
  }

  /// One part's static vertices and triangles as a renderer-space mesh, `z` mirrored and winding turned with it. A
  /// vertex lacking its normal or uvs leaves that attribute short, which packing refuses as malformed.
  fn read_static_mesh(part: &OgfFile) -> Option<VisualMesh> {
    let geometry: &OgfGeometry = part.geometry.as_ref()?;
    let Some(OgfVertexData::Static(vertices)) = &geometry.vertices else {
      return None;
    };
    let mut indices: Vec<u16> = geometry.indices.clone().unwrap_or_default();

    for triangle in indices.as_chunks_mut::<3>().0 {
      triangle.swap(1, 2);
    }

    Some(VisualMesh {
      texture_name: part.texture.as_ref().map(|texture| texture.texture_name.clone()),
      shader_name: part.texture.as_ref().map(|texture| texture.shader_name.clone()),
      positions: vertices
        .iter()
        .map(|vertex| convert_vector(&vertex.position).to_array())
        .collect(),
      normals: vertices
        .iter()
        .map_while(|vertex| vertex.normal.as_ref().map(|normal| convert_vector(normal).to_array()))
        .collect(),
      uvs: vertices.iter().map_while(Self::get_uv).collect(),
      indices,
    })
  }

  /// A static vertex's first uv set, as a renderer binds it.
  fn get_uv(vertex: &OgfStaticVertex) -> Option<[f32; 2]> {
    match vertex.texture_coordinates.first()?.as_slice() {
      [u, v, ..] => {
        let (u, v) = convert_uvs(*u, *v);

        Some([u, v])
      }
      _ => None,
    }
  }
}
