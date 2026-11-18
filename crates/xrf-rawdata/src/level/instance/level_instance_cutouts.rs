use std::collections::HashMap;
use std::path::PathBuf;

use byteorder::ByteOrder;
use glam::{DVec3, Vec3};
use xrf_editable_level::{EditableCustomObject, EditableSceneObject};
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_spawn::{AlifeObject, AlifeObjectInherited, Shape};

use crate::level::build::level_build_part_box::LevelBuildPartBox;
use crate::level::build::level_build_parts::LevelBuildParts;
use crate::level::instance::level_instance_placement::LevelInstancePlacement;

/// The ladders and breakable parts the placed library objects make again when the scene is built: the boxes of the
/// parts their faces of a climbable or a breakable game material make, cut out as the Build cuts them
/// ([`LevelBuildParts`]). A library object keeps the faces the Build turned into a level's ladder or breakable, so a
/// packet such a part matches already comes back with the placement; folded into an object of its own as well, it
/// would be built twice.
#[derive(Default)]
pub(crate) struct LevelInstanceCutouts {
  ladders: Vec<LevelBuildPartBox>,
  breakables: Vec<LevelBuildPartBox>,
}

impl LevelInstanceCutouts {
  /// How far a packet's box may stand, and its half sizes differ, from a part's, metres: a placement is found within
  /// a fraction of a millimetre, and the editor fits a box in single precision, but no two ladders or breakables of
  /// vanilla's levels stand this near.
  const TOLERANCE: f64 = 0.05;

  /// The parts `placements` make, each placed object read from its library file once, its faces' game materials
  /// looked up in `game_materials`.
  ///
  /// # Errors
  ///
  /// Returns an error when a placed object's file does not read, or, as the Build stops on it, a face of it names a
  /// surface or point its object lacks or a game material the library does not hold.
  pub(crate) fn of<T: ByteOrder>(
    placements: &[LevelInstancePlacement],
    game_materials: &GameMtlFile,
  ) -> XrfResult<Self> {
    let mut objects: HashMap<&PathBuf, EditableObjectFile> = HashMap::new();
    let mut ladders: LevelBuildParts = LevelBuildParts::new(GameMtlMaterial::FLAG_CLIMABLE);
    let mut breakables: LevelBuildParts = LevelBuildParts::new(GameMtlMaterial::FLAG_BREAKABLE);

    for placement in placements {
      if !objects.contains_key(&placement.path) {
        objects.insert(
          &placement.path,
          EditableObjectFile::read_from_path::<T, _>(&placement.path)?,
        );
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

  /// Whether `entity` is a ladder or a breakable a placement makes again: a ladder whose box stands as a part's and is
  /// as long and as wide, a breakable standing at a part's box. A ladder's thickness is not compared: the SDK's library
  /// gives some of vanilla's ladders flat, which GSC's own objects built up to 20 cm thick.
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
