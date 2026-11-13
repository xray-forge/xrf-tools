//! A compile's details step on a level directory: what it reads, writes and refuses.

use std::path::{Path, PathBuf};

use glam::Vec3;
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_job::JobHandle;
use xrf_level::{
  BuildCformFace, BuildCformFile, BuildCformRaycastFace, BuildPrjFile, BuildPrjMaterial, BuildPrjName, BuildPrjParams,
  BuildPrjShader, BuildPrjTexture, LevelDetailsFile, LevelLightsFile,
};
use xrf_math::{Aabb, Vector2d, Vector3d};
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};
use xrf_test_utils::GeneratedResources;
use xrf_thm::{ThmFile, ThmFormat, ThmTextureParamChunk};

use crate::tests::detail::{details, lighting, lights, triangles};
use crate::{
  DetailLightBackend, DetailSlotCounts, LevelCompileReport, LevelCompileRequest, LevelCompileStep, LevelCompileStepRun,
  LevelCompiler,
};

/// Game data holding a compiler shader library of one shadow-casting shader, a textures root with one opaque
/// texture's `.thm`, and `levels/test_level` holding what the details step reads: the lighter tests' grid, ground,
/// roof and lights, every face of the one material.
struct Fixture {
  gamedata: PathBuf,
  level: PathBuf,
}

impl Fixture {
  fn new(name: &str) -> XrfResult<Self> {
    let gamedata: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), name))?;
    let level: PathBuf = gamedata.join("levels").join("test_level");
    let mut thumbnail: ThmFile = ThmFile::new_texture();

    std::fs::create_dir_all(&level)?;
    std::fs::create_dir_all(gamedata.join("textures"))?;
    thumbnail.texture_param = Some(ThmTextureParamChunk {
      format: ThmFormat::Dxt1,
      width: 8,
      height: 4,
      ..ThmTextureParamChunk::default()
    });
    thumbnail.write_to_path::<XRayByteOrder, _>(&gamedata.join("textures").join("ground.thm"))?;

    Self::write(&gamedata.join(ShaderCompilerFile::FILE_NAME), |writer| {
      ShaderCompilerFile {
        shaders: vec![ShaderCompilerShader {
          name: String::from("default"),
          // The rest of the name's 128-byte field.
          name_trailing: vec![0; 128 - "default".len()],
          flags: ShaderCompilerShader::CASTS_SHADOW,
          vertex_translucency: 0.5,
          vertex_ambient: 0.0,
          lightmap_density: 1.0,
        }],
      }
      .write::<XRayByteOrder>(writer)
    })?;
    Self::write(&level.join("build.details"), |writer| {
      details().write::<XRayByteOrder>(writer)
    })?;
    Self::write(&level.join(LevelLightsFile::FILE_NAME), |writer| {
      lights().write::<XRayByteOrder>(writer)
    })?;
    Self::write(&level.join(BuildCformFile::FILE_NAME), |writer| {
      Self::cform().write::<XRayByteOrder>(writer)
    })?;
    Self::write(&level.join(BuildPrjFile::FILE_NAME), |writer| {
      Self::project()?.write::<XRayByteOrder>(writer)
    })?;

    Ok(Self { gamedata, level })
  }

  fn request(&self, is_forced: bool) -> LevelCompileRequest {
    LevelCompileRequest {
      level: self.level.clone(),
      steps: vec![LevelCompileStep::Details],
      gamedata: self.gamedata.clone(),
      textures: vec![self.gamedata.join("textures")],
      is_forced,
      // The reference, whatever the build: the light and the report alike on every machine.
      detail_backend: DetailLightBackend::Cpu,
      ..LevelCompileRequest::default()
    }
  }

  fn write(path: &Path, write: impl FnOnce(&mut ChunkWriter) -> XrfResult) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    write(&mut writer)?;
    std::fs::write(path, &writer.buffer)?;

    Ok(())
  }

  fn cform() -> BuildCformFile {
    let triangles: Vec<[Vec3; 3]> = triangles();
    let vertex = |corner: &Vec3| Vector3d::new(corner.x, corner.y, corner.z);

    BuildCformFile {
      version: BuildCformFile::CURRENT_VERSION,
      bounds: Aabb {
        min: Vector3d::new(-1.0, 0.5, -1.0),
        max: Vector3d::new(4.0, 3.0, 3.0),
      },
      vertices: triangles.iter().flatten().map(vertex).collect(),
      faces: (0..triangles.len() as u32)
        .map(|face| BuildCformFace {
          vertices: [face * 3, face * 3 + 1, face * 3 + 2],
          face_handle: face,
        })
        .collect(),
      raycast_faces: triangles
        .iter()
        .map(|_| BuildCformRaycastFace {
          material: 0,
          material_padding: 0,
          game_material: 0,
          texture_coordinates: [Vector2d { x: 0.0, y: 0.0 }; 3],
          reserved: 0,
          trailing_padding: 0,
        })
        .collect(),
    }
  }

  /// One material, the opaque texture under the shadow-casting compiler shader.
  fn project() -> XrfResult<BuildPrjFile> {
    Ok(BuildPrjFile {
      version: BuildPrjFile::CURRENT_VERSION,
      params: BuildPrjParams {
        smooth_angle: 0.0,
        weld_distance: 0.0,
        lightmap_pixels_per_meter: 0.0,
        lightmap_jitter_samples: 0,
        lightmap_rms_zero: 0,
        lightmap_rms: 0,
        quality: 0,
        quality_reserved: 0,
        reserved: [0.0; 6],
      },
      vertices: Vec::new(),
      faces: Vec::new(),
      smoothing_groups: None,
      materials: vec![BuildPrjMaterial {
        texture: 0,
        shader: 0,
        compiler_shader: 0,
        sector: 0,
        reserved: 0,
        area_padding: 0,
        internal_max_area: 0,
      }],
      render_shaders: Vec::new(),
      compiler_shaders: vec![BuildPrjShader {
        name: BuildPrjName::new("default", BuildPrjName::LONG_SIZE)?,
      }],
      textures: vec![BuildPrjTexture {
        name: BuildPrjName::new("ground", BuildPrjName::LONG_SIZE)?,
        width: 0,
        height: 0,
        has_alpha: 0,
        surface: 0,
      }],
      glows: Vec::new(),
      portals: Vec::new(),
      light_controls: Vec::new(),
      static_lights: Vec::new(),
      dynamic_lights: Vec::new(),
      lods: Vec::new(),
      mu_models: Vec::new(),
      mu_references: Vec::new(),
    })
  }
}

#[test]
fn writes_the_details_lit_as_the_lighter_lights_them_with_no_sdk() -> XrfResult {
  let fixture: Fixture = Fixture::new("lit")?;
  let report: LevelCompileReport = LevelCompiler::compile(&fixture.request(false), &JobHandle::inert())?;
  let written: LevelDetailsFile =
    LevelDetailsFile::read_from_path::<XRayByteOrder, _>(&fixture.level.join(LevelDetailsFile::FILE_NAME))?;

  assert_eq!(written, lighting()?.details);
  assert_eq!(report.work, None);
  assert_eq!(
    report
      .steps
      .iter()
      .map(|step| (step.step, &step.run))
      .collect::<Vec<_>>(),
    [(
      LevelCompileStep::Details,
      &LevelCompileStepRun::DetailLighter {
        slots: DetailSlotCounts {
          unplanted: 1,
          emptied: 1,
          untouched: 1,
          lit: 2,
        },
        gpu: None,
      }
    )]
  );

  Ok(())
}

#[test]
fn refuses_to_overwrite_the_details_unless_forced() -> XrfResult {
  let fixture: Fixture = Fixture::new("forced")?;
  let output: PathBuf = fixture.level.join(LevelDetailsFile::FILE_NAME);

  std::fs::write(&output, b"kept")?;

  let error: String = LevelCompiler::compile(&fixture.request(false), &JobHandle::inert())
    .unwrap_err()
    .to_string();

  assert!(error.contains("force the compile"), "{error}");
  assert_eq!(std::fs::read(&output)?, b"kept");

  LevelCompiler::compile(&fixture.request(true), &JobHandle::inert())?;

  assert_eq!(
    LevelDetailsFile::read_from_path::<XRayByteOrder, _>(&output)?,
    lighting()?.details
  );

  Ok(())
}

#[test]
fn refuses_game_data_without_the_compiler_shader_library() -> XrfResult {
  let fixture: Fixture = Fixture::new("no_library")?;

  std::fs::remove_file(fixture.gamedata.join(ShaderCompilerFile::FILE_NAME))?;

  let error: String = LevelCompiler::compile(&fixture.request(false), &JobHandle::inert())
    .unwrap_err()
    .to_string();

  assert!(error.contains(ShaderCompilerFile::FILE_NAME), "{error}");

  Ok(())
}
