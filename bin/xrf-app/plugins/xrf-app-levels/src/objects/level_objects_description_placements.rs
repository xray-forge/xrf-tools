use glam::Vec3;
use xrf_level::{LevelDynamicLight, LevelEnvModifier, LevelFogVolume, LevelSndEnvFile};
use xrf_math::{Matrix4x4, Vector3d};
use xrf_visual::VisualTransform;

use crate::objects::{
  LevelObject, LevelObjectId, LevelObjectKind, LevelObjectLight, LevelObjectPlacement, LevelObjectShape,
  LevelObjectsDescription, LevelPlacementDetails, LevelPlacementFile,
};
use crate::parts::LevelPlacements;

impl LevelObjectsDescription {
  /// `eGameIDSingle`: the bit a planted system's game types hold for `CLevel::Load_GameSpecific_After` to plant it in a
  /// single player game.
  const SINGLE_PLAYER: u16 = 1;

  /// Corners and faces a box of `level.snd_env` takes: the editor writes each volume as a whole box.
  const SOUND_BOX_CORNERS: usize = 8;
  const SOUND_BOX_FACES: usize = 12;

  /// What the level's companion files place, file by file in their own order: the systems planted for a single player
  /// game, the static sounds, the sound environments' boxes, the fog volumes, the environment modifiers and the
  /// respawn points.
  pub(super) fn add_placements(objects: &mut Vec<LevelObject>, placements: &LevelPlacements) {
    let mut add = |(file, index): (LevelPlacementFile, usize),
                   (kind, name): (LevelObjectKind, String),
                   (transform, shapes): (VisualTransform, Vec<LevelObjectShape>),
                   details: LevelPlacementDetails| {
      objects.push(LevelObject {
        placement: Some(LevelObjectPlacement { shapes, details }),
        ..LevelObject::new(
          LevelObjectId::Placement {
            file,
            index: index as u32,
          },
          kind,
          name,
          transform,
        )
      });
    };

    for (index, planted) in placements
      .particles
      .iter()
      .flat_map(|file| file.placements.iter().enumerate())
    {
      if planted.game_types.is_none_or(|types| types & Self::SINGLE_PLAYER != 0) {
        add(
          (LevelPlacementFile::Particles, index),
          (LevelObjectKind::StaticParticles, planted.effect.clone()),
          (Self::to_place(&planted.get_position()), Vec::new()),
          LevelPlacementDetails::Particles {
            effect: planted.effect.clone(),
          },
        );
      }
    }

    for (index, sound) in placements.sounds.iter().flat_map(|file| file.sounds.iter().enumerate()) {
      add(
        (LevelPlacementFile::Sounds, index),
        (LevelObjectKind::StaticSounds, sound.sound.clone()),
        (Self::to_place(&sound.position), Vec::new()),
        LevelPlacementDetails::Sound {
          sound: sound.sound.clone(),
          volume: sound.volume,
          frequency: sound.frequency,
          active: [sound.active.from, sound.active.to],
          play: [sound.play.from, sound.play.to],
          pause: [sound.pause.from, sound.pause.to],
        },
      );
    }

    if let Some(file) = &placements.sound_environments {
      for (index, corners) in file
        .vertices
        .as_chunks::<{ Self::SOUND_BOX_CORNERS }>()
        .0
        .iter()
        .enumerate()
      {
        let Some((center, axes)) = Self::to_corner_box(corners) else {
          continue;
        };
        let (outer, inner): (String, String) = Self::to_sound_environments(file, index);

        add(
          (LevelPlacementFile::SoundEnvironments, index),
          (LevelObjectKind::SoundEnvironments, format!("{outer} / {inner}")),
          (
            Self::to_renderer_place(center),
            vec![LevelObjectShape::Box {
              axes,
              center: Vector3d::new(0.0, 0.0, 0.0),
            }],
          ),
          LevelPlacementDetails::SoundEnvironment { outer, inner },
        );
      }
    }

    for (index, volume) in placements
      .fog_volumes
      .iter()
      .flat_map(|file| file.volumes.iter().enumerate())
    {
      add(
        (LevelPlacementFile::FogVolumes, index),
        (LevelObjectKind::FogVolumes, volume.profile.clone()),
        Self::to_fog_volume(volume),
        LevelPlacementDetails::FogVolume {
          profile: volume.profile.clone(),
          obstacles: volume.obstacles.len() as u32,
        },
      );
    }

    for (index, modifier) in placements
      .env_modifiers
      .iter()
      .flat_map(|file| file.modifiers.iter().enumerate())
    {
      add(
        (LevelPlacementFile::EnvModifiers, index),
        (LevelObjectKind::EnvModifiers, format!("modifier {index}")),
        (
          Self::to_place(&modifier.position),
          vec![LevelObjectShape::Sphere {
            center: Vector3d::new(0.0, 0.0, 0.0),
            radius: modifier.radius,
          }],
        ),
        Self::to_env_modifier(modifier),
      );
    }

    for (index, point) in placements.game.iter().flat_map(|file| file.rpoints.iter().enumerate()) {
      let role: String = point
        .get_kind_label()
        .map_or_else(|| format!("type {}", point.kind), str::to_owned);

      add(
        (LevelPlacementFile::RespawnPoints, index),
        (
          LevelObjectKind::RespawnPoints,
          if point.profile.is_empty() {
            format!("{role}, team {}", point.team)
          } else {
            point.profile.clone()
          },
        ),
        (Self::to_place(&point.position), Vec::new()),
        LevelPlacementDetails::RespawnPoint {
          role,
          team: point.team,
          game_types: point.game_types,
          profile: point.profile.clone(),
        },
      );
    }
  }

  /// The bundle's static lights, the sun aside, which the environment drives rather than the level.
  pub(super) fn add_lights(objects: &mut Vec<LevelObject>, lights: &[LevelDynamicLight]) {
    objects.extend(
      lights
        .iter()
        .enumerate()
        .filter(|(_, light)| !light.is_sun())
        .map(|(index, light)| LevelObject {
          light: Some(LevelObjectLight::from(light)),
          ..LevelObject::new(
            LevelObjectId::Light { index: index as u32 },
            LevelObjectKind::Lights,
            format!("light {index}"),
            Self::to_place(&light.position),
          )
        }),
    );
  }

  /// The environments outside and inside a sound box, by its first face's front and back, as the editor writes them.
  fn to_sound_environments(file: &LevelSndEnvFile, index: usize) -> (String, String) {
    let face = file.faces.get(index * Self::SOUND_BOX_FACES);
    let name = |name: Option<&str>| name.unwrap_or("unnamed").to_owned();

    (
      name(face.and_then(|face| file.get_front_environment(face))),
      name(face.and_then(|face| file.get_back_environment(face))),
    )
  }

  /// A box's centre and its three edges in renderer space, from its eight corners in engine space: the two shortest
  /// edges from a corner, and the corner standing most squarely off both; `None` for corners spanning no box.
  fn to_corner_box(corners: &[Vector3d]) -> Option<(Vec3, [Vector3d; 3])> {
    let points: Vec<Vec3> = corners
      .iter()
      .map(|corner| Self::to_vec3(&LevelObject::to_renderer_space(corner)))
      .collect();
    let first: Vec3 = *points.first()?;
    let mut spans: Vec<Vec3> = points.iter().skip(1).map(|point| *point - first).collect();

    spans.sort_by(|a, b| a.length_squared().total_cmp(&b.length_squared()));

    let [a, b]: [Vec3; 2] = [*spans.first()?, *spans.get(1)?];
    let normal: Vec3 = a.cross(b).normalize_or_zero();
    let c: Vec3 = spans.iter().skip(2).copied().max_by(|left, right| {
      let squareness = |span: &Vec3| span.dot(normal).abs() / span.length().max(f32::EPSILON);

      squareness(left).total_cmp(&squareness(right))
    })?;

    (normal != Vec3::ZERO && c.length() > f32::EPSILON).then(|| {
      (
        points.iter().copied().sum::<Vec3>() / points.len() as f32,
        [a, b, c].map(|axis| Vector3d::new(axis.x, axis.y, axis.z)),
      )
    })
  }

  /// A fog volume: the unit cube its matrix stands, and each obstacle's cube cutting it, about the volume's centre.
  fn to_fog_volume(volume: &LevelFogVolume) -> (VisualTransform, Vec<LevelObjectShape>) {
    let center: Vec3 = Self::to_vec3(&LevelObject::to_renderer_space(&Self::get_matrix_row(
      &volume.transform,
      3,
    )));
    let to_box = |matrix: &Matrix4x4| -> LevelObjectShape {
      let at: Vec3 = Self::to_vec3(&LevelObject::to_renderer_space(&Self::get_matrix_row(matrix, 3)));

      LevelObjectShape::Box {
        axes: [0, 1, 2].map(|row| LevelObject::to_renderer_space(&Self::get_matrix_row(matrix, row))),
        center: Vector3d::new(at.x - center.x, at.y - center.y, at.z - center.z),
      }
    };

    (
      Self::to_renderer_place(center),
      std::iter::once(&volume.transform)
        .chain(&volume.obstacles)
        .map(to_box)
        .collect(),
    )
  }

  fn to_env_modifier(modifier: &LevelEnvModifier) -> LevelPlacementDetails {
    let to_array = |vector: &Vector3d| [vector.x, vector.y, vector.z];

    LevelPlacementDetails::EnvModifier {
      power: modifier.power,
      far_plane: modifier.far_plane,
      fog_color: to_array(&modifier.fog_color),
      fog_density: modifier.fog_density,
      ambient: to_array(&modifier.ambient),
      sky_color: to_array(&modifier.sky_color),
      hemi_color: to_array(&modifier.hemi_color),
      affects: modifier.get_used_parameters().into_iter().map(str::to_owned).collect(),
    }
  }

  /// One of an `Fmatrix`'s rows, `i`, `j`, `k` or `c` by index.
  fn get_matrix_row(matrix: &Matrix4x4, row: usize) -> Vector3d {
    let at: usize = row * 4;

    Vector3d::new(matrix.values[at], matrix.values[at + 1], matrix.values[at + 2])
  }

  /// Unturned at a point already in renderer space.
  fn to_renderer_place(center: Vec3) -> VisualTransform {
    VisualTransform {
      c: Vector3d::new(center.x, center.y, center.z),
      ..Self::to_place(&Vector3d::new(0.0, 0.0, 0.0))
    }
  }

  fn to_vec3(vector: &Vector3d) -> Vec3 {
    Vec3::new(vector.x, vector.y, vector.z)
  }
}
