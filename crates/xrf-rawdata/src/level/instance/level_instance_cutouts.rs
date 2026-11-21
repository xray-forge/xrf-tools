use std::collections::HashMap;
use std::path::PathBuf;

use glam::{DVec3, Vec3};
use xrf_editable_level::{EditableCustomObject, EditableSceneObject};
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_spawn::{AlifeObject, AlifeObjectInherited, Shape};

use crate::level::build::level_build_part_box::LevelBuildPartBox;
use crate::level::build::level_build_parts::LevelBuildParts;
use crate::level::instance::level_instance_placement::LevelInstancePlacement;

/// The boxes of the ladders and breakable parts the placed library objects make again when the scene is built, cut as
/// the Build cuts them ([`LevelBuildParts`]).
///
/// A packet matching one already comes back with its placement; folded into an object of its own too, it would be
/// built twice.
#[derive(Default)]
pub(crate) struct LevelInstanceCutouts {
  ladders: Vec<LevelBuildPartBox>,
  breakables: Vec<LevelBuildPartBox>,
}

impl LevelInstanceCutouts {
  /// How far a packet's box centre and half sizes may differ from a part's, metres: well above placement and
  /// single-precision error, and below the gap between any two vanilla ladders or breakables.
  const TOLERANCE: f64 = 0.05;

  /// The parts `placements` make, each placed object's file read once.
  ///
  /// # Errors
  ///
  /// Returns an error when a placed object's file does not read, or, as in the Build, a face names a surface, point or
  /// game material that does not exist.
  pub(crate) fn of(placements: &[LevelInstancePlacement], game_materials: &GameMtlFile) -> XrfResult<Self> {
    let mut objects: HashMap<&PathBuf, EditableObjectFile> = HashMap::new();
    let mut ladders: LevelBuildParts = LevelBuildParts::new(GameMtlMaterial::FLAG_CLIMABLE);
    let mut breakables: LevelBuildParts = LevelBuildParts::new(GameMtlMaterial::FLAG_BREAKABLE);

    for placement in placements {
      if !objects.contains_key(&placement.path) {
        objects.insert(&placement.path, EditableObjectFile::read_from_path(&placement.path)?);
      }

      let object: &EditableObjectFile = &objects[&placement.path];
      let placed: EditableSceneObject = EditableSceneObject {
        object: EditableCustomObject::new(placement.object.clone(), placement.transform),
        reference: placement.reference.clone(),
        flags: 0,
      };

      ladders.add_object(&placed, object, game_materials)?;
      breakables.add_object(&placed, object, game_materials)?;
    }

    Ok(Self {
      ladders: Self::get_boxes(&ladders),
      breakables: Self::get_boxes(&breakables),
    })
  }

  /// Whether `entity` is a ladder or breakable a placement makes again: a ladder matching a part's box but for its
  /// thickness, or a breakable standing at a part's box.
  ///
  /// Thickness is skipped because the SDK's library has some vanilla ladders flat that GSC built up to 20 cm thick.
  pub(crate) fn is_made(&self, entity: &AlifeObject) -> bool {
    let position: DVec3 = Vec3::from(entity.header.position).as_dvec3();

    match &entity.inherited {
      AlifeObjectInherited::CseAlifeObjectClimable(climable) => {
        let [Shape::Box(shape)] = climable.base.shape.as_slice() else {
          return false;
        };
        let half: [f64; 3] =
          Self::sorted([shape.x_axis, shape.y_axis, shape.z_axis].map(|axis| f64::from(axis.length())));

        self.ladders.iter().any(|part| {
          Self::is_near(part.centre, position) && Self::is_near_outline(&Self::sorted(part.half.to_array()), &half)
        })
      }
      AlifeObjectInherited::CseAlifeObjectBreakable(_) => {
        self.breakables.iter().any(|part| Self::is_near(part.centre, position))
      }
      _ => false,
    }
  }

  /// Each part's box; a part whose corners hold none makes no entity.
  fn get_boxes(parts: &LevelBuildParts) -> Vec<LevelBuildPartBox> {
    parts
      .get_parts()
      .iter()
      .filter_map(|part| parts.fit_box(part))
      .collect()
  }

  fn is_near(first: DVec3, second: DVec3) -> bool {
    (first - second).abs().cmple(DVec3::splat(Self::TOLERANCE)).all()
  }

  /// Whether two boxes' sorted half sizes, thinnest first, agree but for the thinnest.
  fn is_near_outline(first: &[f64; 3], second: &[f64; 3]) -> bool {
    first[1..]
      .iter()
      .zip(&second[1..])
      .all(|(first, second)| (first - second).abs() <= Self::TOLERANCE)
  }

  fn sorted(mut values: [f64; 3]) -> [f64; 3] {
    values.sort_unstable_by(f64::total_cmp);
    values
  }
}
