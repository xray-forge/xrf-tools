use std::sync::Arc;
use std::time::Instant;

use xrf_anm::AnmFile;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionSnapshot;
use xrf_app_render::{ResolvedTexture, TextureResolver};
use xrf_chunk::{InMemoryChunkDataSource, XRayByteOrder};
use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelGeomSource, LevelSector, LevelSectorComposition};
use xrf_ltx::Ltx;
use xrf_material::{XrayReducedTextures, XraySurfaceDescriptor};
use xrf_particles::{ParticleCollider, ParticleEngineRules};
use xrf_renderer::{
  RenderAssetSource, RenderLevelDetails, RenderLevelHelpers, RenderLevelParticles, RenderLevelSource, RenderLevelSpawn,
  RenderLoadFailure, RenderSpawnLighting, RenderSpawnModel, RenderSpawnModels,
};
use xrf_vfs::XraySearch;
use xrf_visual::{LightsDescription, SectorAttributes, SectorPackage, SectorPacker, VisualPoser};
use xrf_world::{WorldLevelSource, WorldLevelWeather};

use crate::level::SelectedLevel;
use crate::objects::LevelObjectsDescription;
use crate::parts::{
  LevelCollisionPart, LevelDetailsPart, LevelEnvironment, LevelEnvironmentPart, LevelGeometryPart, LevelHemiPart,
  LevelLightsPart, LevelObjectMotionsPart, LevelParticlesPart, LevelSectionsPart, LevelSpawnPart, LevelSpawnVisual,
  LevelSurfacesPart, LevelTextureReference, LevelTexturesPart, LevelWeatherPart, PackedLevelDetails, PackedLevelLights,
  PackedLevelParticles, SpawnVisualReader,
};
use crate::render::{LevelParticleCollider, LevelRenderParts};
use crate::report::LevelReport;
use crate::weather::LevelWorldWeather;

/// A level's shader table as it is drawn, and the textures its surfaces bind.
struct DrawnSurfaces {
  surfaces: Arc<Vec<XraySurfaceDescriptor>>,
  textures: Arc<Vec<LevelTextureReference>>,
}

/// The open level as the world plays it and the native renderer draws it: sectors packed when its loader asks, textures
/// read from the roots the level was opened in. How its session parts feed the scene is `LevelRenderParts`'.
pub struct LevelRenderSource {
  level: Arc<SessionSnapshot<SelectedLevel>>,
  assets: AssetMountState,
  /// How the level's shader table is drawn, as its surfaces part holds it for the snapshot drawn.
  surfaces: Arc<Vec<XraySurfaceDescriptor>>,
  /// Where each texture reference the level's surfaces bind resolved to at open, each projector its lights name once
  /// they are read, and anything else once asked for, as the weather's skies are; shared by the session's next
  /// sources, which keep its resolutions, a texture's file come since being loaded afresh by the texture watcher.
  textures: Arc<TextureResolver>,
  /// The textures the game loads a step coarser at the same texture quality, out of the sections read as it opened.
  reduced: XrayReducedTextures,
}

impl LevelRenderSource {
  /// What the viewer draws a level surface with, which is what a pack is worth carrying: the tangent frame too, whose
  /// fourth bytes are the low bytes of the base coordinate, and the vertex colour water is lit by.
  const DRAWN_ATTRIBUTES: SectorAttributes = SectorAttributes {
    binormals: true,
    colors: true,
    lightmap_uvs: true,
    normals: true,
    tangents: true,
    uvs: true,
  };

  /// The level `level` holds as the renderer draws it, its surfaces and textures as its parts hold them.
  pub fn new(level: Arc<SessionSnapshot<SelectedLevel>>, assets: AssetMountState) -> Self {
    let DrawnSurfaces { surfaces, textures } = Self::find_surfaces(&level, &assets);
    let resolver: TextureResolver = TextureResolver::new(
      level.roots.clone(),
      level.source.get_texture_scope(),
      textures.iter().map(ResolvedTexture::from),
    );

    Self {
      reduced: Self::find_reduced(&level).unwrap_or_default(),
      surfaces,
      level,
      assets,
      textures: Arc::new(resolver),
    }
  }

  /// The level's surfaces and their textures as its parts hold them, resolved where a commit let them go; none where
  /// its roots cannot be searched, which draws every surface flat.
  fn find_surfaces(level: &SessionSnapshot<SelectedLevel>, assets: &AssetMountState) -> DrawnSurfaces {
    if let (Some(surfaces), Some(textures)) = (LevelSurfacesPart::find(level), LevelTexturesPart::find(level)) {
      return DrawnSurfaces { surfaces, textures };
    }

    match assets.search(&level.roots) {
      Ok(search) => DrawnSurfaces {
        surfaces: LevelSurfacesPart::get(level, &search),
        textures: LevelTexturesPart::get(level, &search),
      },
      Err(error) => {
        LevelReport::unresolved_surfaces(&level.source, &error);

        DrawnSurfaces {
          surfaces: Arc::default(),
          textures: Arc::default(),
        }
      }
    }
  }

  /// The textures the sections the level holds already reduce, so no texture load waits on the configs; none where
  /// they are not read, or could not be.
  fn find_reduced(level: &SessionSnapshot<SelectedLevel>) -> Option<XrayReducedTextures> {
    LevelSectionsPart::find(level)
      .as_ref()
      .and_then(|sections| sections.section(XrayReducedTextures::SECTION))
      .map(|section| XrayReducedTextures::new(section.iter().map(|(key, _)| key)))
  }

  /// The same level drawn from `next`, a later snapshot of its session sharing its value and its surfaces, with the
  /// textures resolved so far, and the reduced ones known until `next` holds sections of its own.
  pub fn follow(&self, next: Arc<SessionSnapshot<SelectedLevel>>) -> Self {
    Self {
      reduced: Self::find_reduced(&next).unwrap_or_else(|| self.reduced.clone()),
      surfaces: LevelSurfacesPart::find(&next).unwrap_or_else(|| Arc::clone(&self.surfaces)),
      level: next,
      assets: AssetMountState::clone(&self.assets),
      textures: Arc::clone(&self.textures),
    }
  }

  /// The snapshot it draws.
  pub fn get_level(&self) -> &Arc<SessionSnapshot<SelectedLevel>> {
    &self.level
  }

  /// Whether it reads its textures through the same resolver as `other`, as a source following another does.
  #[cfg(test)]
  pub fn shares_textures_with(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.textures, &other.textures)
  }

  /// A search over the roots the level opened with, which every read the renderer asks for goes through.
  fn search(&self) -> XrfResult<XraySearch> {
    self.assets.search(&self.level.roots).map_err(XrfError::new_asset_error)
  }

  /// The sections a level's spawned objects name, none where its spawn or the configs cannot be read.
  fn read_sections(level: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Option<Arc<Ltx>> {
    LevelSectionsPart::get(level, search)
      .inspect_err(LevelReport::missing_sections)
      .ok()
  }
}

impl RenderAssetSource for LevelRenderSource {
  fn get_texture_scope(&self) -> &str {
    self.textures.get_scope_key()
  }

  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    self.textures.read(&self.assets, reference)
  }

  fn forget_textures(&self, names: &[String]) {
    self.textures.forget(names);
  }

  fn get_texture_stamp(&self, reference: &str) -> Option<u64> {
    self.textures.get_stamp(&self.assets, reference)
  }

  fn is_texture_reduced(&self, reference: &str) -> bool {
    self.reduced.is_reduced(reference)
  }
}

impl RenderLevelSource for LevelRenderSource {
  fn get_sector_count(&self) -> u32 {
    self.level.get_sectors().len() as u32
  }

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &self.surfaces
  }
}

impl WorldLevelSource for LevelRenderSource {
  /// The level's session: every source of one opening shares a scene, its next snapshots' included, and a level opened
  /// again streams afresh.
  fn get_scene_key(&self) -> Option<String> {
    Some(LevelRenderParts::to_scene_key(self.level.session_id))
  }

  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage> {
    let sectors: &[LevelSector] = self.level.get_sectors();
    let root: u32 = sectors
      .get(sector as usize)
      .ok_or_else(|| XrfError::new_not_found_error(format!("The level has no sector {sector}")))?
      .root;
    let composition: LevelSectorComposition = LevelSectorComposition::of(&self.level.visuals, root);
    let geometry: Arc<LevelGeomSource<InMemoryChunkDataSource>> =
      LevelGeometryPart::get(&self.level, &self.search()?).map_err(XrfError::new_read_error)?;
    let started: Instant = Instant::now();
    let package: SectorPackage = SectorPacker::new(&self.level.visuals, self.level.level.shaders.as_ref(), &geometry)
      .pack::<XRayByteOrder>(sector, &composition, Self::DRAWN_ATTRIBUTES);

    LevelReport::packed_sector(&package, started);

    Ok(package)
  }

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    let level: &SessionSnapshot<SelectedLevel> = &self.level;
    let search: XraySearch = self.search()?;
    let packed: PackedLevelLights =
      LevelLightsPart::pack(level, &search, Self::read_sections(level, &search).as_deref());

    self
      .textures
      .extend(packed.projectors.iter().map(ResolvedTexture::from));

    Ok(packed.lights)
  }

  fn read_weather(&self) -> XrfResult<WorldLevelWeather> {
    let level: &SessionSnapshot<SelectedLevel> = &self.level;
    let search: XraySearch = self.search()?;
    let environment: Arc<LevelEnvironment> =
      LevelEnvironmentPart::get(level, &search).map_err(XrfError::new_asset_error)?;

    Ok(LevelWeatherPart::record(level, &search, |search| {
      LevelWorldWeather::read(level, &environment, search)
    }))
  }

  fn read_weather_cycle(&self, name: &str) -> XrfResult<Vec<WeatherDescriptor>> {
    let environment: Arc<LevelEnvironment> =
      LevelEnvironmentPart::get(&self.level, &self.search()?).map_err(XrfError::new_asset_error)?;
    let cycle = environment
      .catalog
      .find_cycle(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("There is no weather cycle '{name}'")))?;

    Ok(cycle.list_descriptors(environment.catalog.engine))
  }

  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
    let packed: Option<PackedLevelDetails> =
      LevelDetailsPart::pack(&self.level, &self.search()?).map_err(XrfError::new_asset_error)?;
    let Some(packed) = packed else {
      return Ok(None);
    };

    self.textures.extend(packed.textures.iter().map(ResolvedTexture::from));

    Ok(Some(RenderLevelDetails {
      package: packed.package,
      surfaces: packed.surfaces,
    }))
  }

  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn> {
    let spawn = LevelSpawnPart::get(&self.level, &self.search()?).map_err(XrfError::new_asset_error)?;

    Ok(RenderLevelSpawn::from(&*spawn))
  }

  fn read_helpers(&self) -> XrfResult<RenderLevelHelpers> {
    // The objects the page lists, in its order: each helper is named by its object's place among them.
    let objects: LevelObjectsDescription =
      LevelObjectsDescription::read(&self.level, &self.search()?).map_err(XrfError::new_asset_error)?;

    Ok(RenderLevelHelpers::from(&objects))
  }

  fn read_particles(&self) -> XrfResult<Option<RenderLevelParticles>> {
    let level: &SessionSnapshot<SelectedLevel> = &self.level;
    let search: XraySearch = self.search()?;
    let packed: PackedLevelParticles =
      LevelParticlesPart::pack(level, &search, Self::read_sections(level, &search).as_deref())
        .map_err(XrfError::new_asset_error)?;
    let collision = LevelCollisionPart::get(level, &search);
    let hemi = LevelHemiPart::get(level, &search);

    self.textures.extend(packed.textures.iter().map(ResolvedTexture::from));

    Ok(Some(RenderLevelParticles {
      library: Arc::new(packed.library),
      rules: ParticleEngineRules::new(level.engine.engine, ParticleEngineRules::DEFAULT_UPDATE_COEFFICIENT),
      placements: packed.placements,
      surfaces: packed.surfaces,
      collider: collision
        .inspect_err(|error| LevelReport::missing_particle_collision(&level.source, error))
        .ok()
        .map(|tracer| Arc::new(LevelParticleCollider::new(tracer)) as Arc<dyn ParticleCollider>),
      // Its failure is reported where the estimator is built.
      hemi: hemi.ok(),
    }))
  }

  fn read_object_motion(&self, name: &str) -> XrfResult<AnmFile> {
    LevelObjectMotionsPart::read(&self.level, &self.search()?, name).map_err(XrfError::new_asset_error)
  }

  fn read_spawn_models(&self, names: &[String]) -> XrfResult<RenderSpawnModels> {
    let level: &SessionSnapshot<SelectedLevel> = &self.level;
    let search: XraySearch = self.search()?;
    let mut failures: Vec<RenderLoadFailure> = Vec::new();
    // A visual that cannot be read is reported by the reader, and left out.
    let read: Vec<(&str, Arc<LevelSpawnVisual>)> = SpawnVisualReader::with(level, &search, |visuals| {
      names
        .iter()
        .filter_map(|name| match visuals.get(name) {
          Ok(visual) => Some((name.as_str(), visual)),
          Err(error) => {
            failures.push(RenderLoadFailure {
              name: name.clone(),
              reason: error.to_string(),
            });

            None
          }
        })
        .collect()
    });
    let hemi = LevelHemiPart::get(level, &search)
      .map(|estimator| LevelHemiPart::estimate_visuals(level, &search, &estimator, &read))
      .unwrap_or_default();

    self.textures.extend(
      read
        .iter()
        .flat_map(|(_, visual)| visual.textures.iter().map(ResolvedTexture::from)),
    );

    Ok(RenderSpawnModels {
      models: read
        .iter()
        .map(|(name, visual)| RenderSpawnModel {
          name: (*name).to_owned(),
          package: VisualPoser::pose(&visual.package, visual.rest.as_deref()),
          surfaces: visual.surfaces.clone(),
          skeleton: None,
        })
        .collect(),
      lighting: hemi
        .into_iter()
        .map(|object| RenderSpawnLighting {
          object: object.index,
          cube: object.cube,
          sky: object.sky,
        })
        .collect(),
      failures,
    })
  }
}
