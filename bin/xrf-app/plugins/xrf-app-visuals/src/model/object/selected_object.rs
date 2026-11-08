use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetSource;
use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_extension::XrayExtension;
use xrf_ogf::OgfFile;
use xrf_rawdata::{ObjectCompileOptions, ObjectCompileSource, ObjectCompiler, ObjectProvenance};
use xrf_vfs::XraySearch;

use crate::model::object::selected_object_description::SelectedObjectDescription;

/// An SDK `.object` opened in the viewer: compiled in memory as `xrf-cli object compile` would write it, so it is shown,
/// posed and described as any model is, and described further for what only the object holds.
pub struct SelectedObject;

impl SelectedObject {
  /// Whether `source` names an SDK object rather than a compiled model.
  pub fn is_object(source: &AssetSource) -> bool {
    XrayExtension::Object.matches(source.label())
  }

  /// The object `source` names, compiled, with what it alone holds described.
  ///
  /// # Errors
  ///
  /// Returns an error when the object cannot be read or does not compile.
  pub fn read(source: &AssetSource, search: &XraySearch) -> TauriResult<(OgfFile, SelectedObjectDescription)> {
    let label: &str = source.label();
    let object: EditableObjectFile = source
      .read(search)
      .and_then(EditableObjectFile::read_from_bytes::<XRayByteOrder>)
      .map_err(|error| format!("Failed to read object '{label}': {error}"))?;
    let provenance: Option<ObjectProvenance> = source.physical_path().and_then(|path| {
      ObjectProvenance::read_beside(path)
        .inspect_err(|error| log::warn!("Object '{label}' has a sidecar that does not read: {error}"))
        .ok()
        .flatten()
    });
    // A decompiled object compiles back with the tangent method its source was made with, as the CLI's default does.
    let file: OgfFile = ObjectCompiler::compile_with(
      &object,
      &ObjectCompileSource {
        source_file: String::from(label),
        convertor: String::from("xrf-app visuals"),
        built_at: 0,
      },
      &ObjectCompileOptions::following(provenance.as_ref()),
    )
    .and_then(|compiled| OgfFile::read_from_bytes::<XRayByteOrder>(compiled.ogf))
    .map_err(|error| format!("Failed to compile object '{label}': {error}"))?;

    Ok((file, SelectedObjectDescription::of(&object, provenance.as_ref())))
  }
}
