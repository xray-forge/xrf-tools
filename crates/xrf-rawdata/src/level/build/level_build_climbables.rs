use xrf_editable_level::EditableSceneObject;
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_math::glam::{DVec3, Vec3};
use xrf_math::{Affine3d, Vector3d};
use xrf_spawn::{
  AlifeObject, AlifeObjectAbstract, AlifeObjectClimable, AlifeObjectInherited, AlifeObjectShape, ClsId, Shape,
  SpawnPacketHeader,
};

use crate::level::build::level_build_climbable_export::LevelBuildClimbableExport;
use crate::level::build::level_build_part_box::LevelBuildPartBox;
use crate::level::build::level_build_part_groups::LevelBuildPartGroups;
use crate::level::build::level_build_parts::LevelBuildParts;

/// The ladders a build cuts out of the geometry, `ESceneObjectTool::ExportClimableObjects`.
///
/// Each part of climbable faces ([`LevelBuildParts`]) becomes a `climable_object` `clmbl#<n>` shaped as its box, turned
/// so its thinnest axis faces the way its faces do (`OrientToNorm`), and of its last face's game material.
pub(crate) struct LevelBuildClimbables {
  parts: LevelBuildParts,
}

impl Default for LevelBuildClimbables {
  fn default() -> Self {
    Self {
      parts: LevelBuildParts::new(GameMtlMaterial::FLAG_CLIMABLE),
    }
  }
}

impl LevelBuildClimbables {
  /// The class every ladder is of, and what names it.
  const SECTION: &'static str = "climable_object";
  const NAME_PREFIX: &'static str = "clmbl#";

  /// Adds every face of a climbable game material of `object`, placed by `placed` ([`LevelBuildParts::add_object`]).
  ///
  /// # Errors
  ///
  /// As [`LevelBuildParts::add_object`].
  pub(crate) fn add_object(
    &mut self,
    placed: &EditableSceneObject,
    object: &EditableObjectFile,
    game_materials: &GameMtlFile,
  ) -> XrfResult {
    self.parts.add_object(placed, object, game_materials)
  }

  /// A ladder of each part, numbered by part. As the Build does, it refuses a part with a face sharing no edge with
  /// another, or whose thinnest axis is not the one its faces face along.
  pub(crate) fn into_export(self) -> LevelBuildClimbableExport {
    let mut export: LevelBuildClimbableExport = LevelBuildClimbableExport {
      entities: Vec::new(),
      refused: Vec::new(),
    };

    let groups: LevelBuildPartGroups = self.parts.group();

    for (index, part) in groups.parts.iter().enumerate() {
      let name: String = format!("{}{index}", Self::NAME_PREFIX);

      match self.get_entity(&groups, &name, part) {
        Ok(entity) => export.entities.push(entity),
        Err(reason) => export.refused.push((name, reason)),
      }
    }

    export
  }

  /// The ladder `part` makes, or why it makes none.
  fn get_entity(&self, groups: &LevelBuildPartGroups, name: &str, part: &[usize]) -> Result<AlifeObject, String> {
    if groups
      .get_edge_neighbours(&self.parts.faces, part)
      .iter()
      .any(Vec::is_empty)
    {
      return Err(String::from(
        "a face of it shares no edge with another (\"Error face found\")",
      ));
    }

    let mut fitted: LevelBuildPartBox = self
      .parts
      .fit_box(part)
      .ok_or_else(|| String::from("its corners hold no box"))?;
    let normal: DVec3 = part
      .iter()
      .flat_map(|face| self.parts.faces[*face].normals)
      .fold(DVec3::ZERO, |sum, normal| sum + Vec3::from(normal).as_dvec3());

    if !Self::orient_to_normal(fitted.get_local(normal), &mut fitted) {
      return Err(String::from(
        "its thinnest axis is not the one its faces face along most (\"Invalid climable object found\")",
      ));
    }

    let frame: Affine3d = fitted.get_frame();
    let half: Vector3d = Vector3d::from(fitted.half.as_vec3());

    Ok(AlifeObject {
      header: SpawnPacketHeader {
        id: u16::MAX,
        net_action: 1,
        section: String::from(Self::SECTION),
        name: name.to_owned(),
        script_game_id: 0,
        script_rp: 0xFE,
        position: frame.translation,
        direction: frame.get_xyz(),
        respawn_time: 0,
        parent_id: u16::MAX,
        phantom_id: u16::MAX,
        script_flags: 33,
        version: 128,
        game_type: u16::MAX,
        script_version: 12,
        client_data: Vec::new(),
        spawn_id: u16::MAX,
      },
      clsid: ClsId::OClmbl,
      inherited: AlifeObjectInherited::CseAlifeObjectClimable(Box::new(AlifeObjectClimable {
        base: AlifeObjectShape {
          base: AlifeObjectAbstract {
            game_vertex_id: u16::MAX,
            distance: 0.0,
            direct_control: 1,
            level_vertex_id: u32::MAX,
            flags: 0xFFFF_FFBF,
            custom_data: String::new(),
            story_id: u32::MAX,
            spawn_story_id: u32::MAX,
          },
          shape: vec![Shape::Box(Affine3d {
            x_axis: Vector3d::new(half.x, 0.0, 0.0),
            y_axis: Vector3d::new(0.0, half.y, 0.0),
            z_axis: Vector3d::new(0.0, 0.0, half.z),
            translation: Vector3d::ZERO,
          })],
        },
        game_material: part
          .last()
          .map(|face| {
            self.parts.surfaces[self.parts.faces[*face].surface]
              .game_material
              .clone()
          })
          .unwrap_or_default(),
      })),
    })
  }

  /// `OrientToNorm`: false unless the axis the summed normal `local`, in the box's frame, leans on most is the box's
  /// thinnest. Where it leans the negative way, that axis and the next are flipped, keeping the box right-handed.
  fn orient_to_normal(local: DVec3, fitted: &mut LevelBuildPartBox) -> bool {
    let (mut most, mut thinnest) = (0, 0);

    for axis in 1..3 {
      if local[axis].abs() > local[most].abs() {
        most = axis;
      }

      if fitted.half[axis] < fitted.half[thinnest] {
        thinnest = axis;
      }
    }

    if most != thinnest {
      return false;
    }

    if local[most] < 0.0 {
      for axis in [most, (most + 1) % 3] {
        fitted.axes[axis] = -fitted.axes[axis];
      }
    }

    true
  }
}
