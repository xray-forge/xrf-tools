use std::collections::HashMap;
use std::sync::Arc;

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionSnapshot;
use xrf_audio::{AudioMaterial, AudioSceneGeometry, AudioSceneMesh};
use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::{GameMtlAcoustics, GameMtlFile, GameMtlMaterial};
use xrf_level::{LevelCformFace, LevelCformGeometry};
use xrf_vfs::XraySearch;

use crate::level::SelectedLevel;
use crate::parts::LevelCollisionPart;
use crate::report::LevelReport;

/// What shapes a shown level's sound: its collision form's triangles, each with the acoustics of the game material it
/// is made of (`SGameMtl::MtlAcoustics`, or the engine's predefined set by name), read when the audio's simulation asks,
/// on its thread. A face things walk through, as bushes are, lets sound through and is left out.
pub(crate) struct LevelSoundGeometry {
  level: Arc<SessionSnapshot<SelectedLevel>>,
  assets: AssetMountState,
}

impl LevelSoundGeometry {
  pub(crate) fn new(level: Arc<SessionSnapshot<SelectedLevel>>, assets: AssetMountState) -> Self {
    Self { level, assets }
  }

  /// The game materials by their ids, none where the library cannot be read, every face then taking the default.
  fn read_materials(search: &XraySearch) -> HashMap<u16, GameMtlMaterial> {
    match search
      .read(GameMtlFile::FILE_NAME)
      .and_then(GameMtlFile::read_from_bytes::<XRayByteOrder>)
    {
      Ok(library) => library
        .materials
        .into_iter()
        .filter_map(|material| Some((u16::try_from(material.id).ok()?, material)))
        .collect(),
      Err(error) => {
        LevelReport::unreadable_materials(GameMtlFile::FILE_NAME, &error);

        HashMap::new()
      }
    }
  }

  fn to_material(acoustics: GameMtlAcoustics) -> AudioMaterial {
    AudioMaterial {
      absorption: acoustics.absorption,
      scattering: acoustics.scattering,
      transmission: acoustics.transmission,
    }
  }
}

impl AudioSceneGeometry for LevelSoundGeometry {
  fn read_mesh(&self) -> XrfResult<AudioSceneMesh> {
    let search: XraySearch = self
      .assets
      .search(&self.level.roots)
      .map_err(XrfError::new_asset_error)?;
    let geometry: LevelCformGeometry =
      LevelCollisionPart::read_geometry(&self.level.source, &search).map_err(XrfError::new_asset_error)?;
    let library: HashMap<u16, GameMtlMaterial> = Self::read_materials(&search);
    // Each game material a kept face names, by the index the mesh gives it.
    let mut indices: HashMap<u16, u32> = HashMap::new();
    let mut mesh: AudioSceneMesh = AudioSceneMesh {
      vertices: geometry
        .get_vertices()
        .iter()
        .map(|vertex| [vertex.x, vertex.y, vertex.z])
        .collect(),
      ..AudioSceneMesh::default()
    };

    for face in geometry.get_faces() {
      let material: Option<&GameMtlMaterial> = library.get(&face.material);

      if material.is_some_and(GameMtlMaterial::is_passable) {
        continue;
      }

      let LevelCformFace { vertices, .. } = face;
      let index: u32 = *indices.entry(face.material).or_insert_with(|| {
        mesh
          .materials
          .push(material.map_or_else(AudioMaterial::default, |material| {
            Self::to_material(material.get_acoustics())
          }));

        mesh.materials.len() as u32 - 1
      });

      mesh.triangles.push(*vertices);
      mesh.material_indices.push(index);
    }

    log::info!(
      "The level's sound is shaped by {} of its collision form's {} triangles, in {} materials",
      mesh.triangles.len(),
      geometry.get_faces().len(),
      mesh.materials.len()
    );

    Ok(mesh)
  }
}
