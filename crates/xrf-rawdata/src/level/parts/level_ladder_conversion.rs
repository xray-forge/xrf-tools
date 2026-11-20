use xrf_editable_level::EditableObjectTransform;
use xrf_error::XrfResult;
use xrf_math::{Affine3d, Vector3d};
use xrf_spawn::{AlifeObject, AlifeObjectInherited, Shape};

use crate::level::entities::level_entity_conversion::LevelEntityConversion;
use crate::level::geometry::level_decompiled_object::LevelDecompiledObject;
use crate::level::geometry::level_visual_geometry::LevelVisualGeometry;
use crate::level::parts::level_decompiled_ladder::LevelDecompiledLadder;
use crate::object::decompile::geometry::source::surface_dress::SurfaceDress;
use crate::object::decompile::object_decompiler::ObjectDecompiler;

/// A level's ladders folded back into the geometry they were built from.
///
/// The Build exports each climbable part as a `climable_object` of the part's box, its thinnest axis facing the way
/// the faces did, with the game material from packet version 127 (`ESceneObjectTool::ExportClimableObjects`). The
/// authored faces are lost, so a ladder comes back as its box less the back face, which the Build turns into the same
/// box facing the same way. Only 106 of vanilla's 380 ladders are flat, so a quad alone would lose the rest's
/// thickness.
pub struct LevelLadderConversion;

impl LevelLadderConversion {
  /// How GSC dressed its own scenes' ladders (`labx8.object`); the Build reads only the game material.
  pub const TEXTURE: &'static str = "prop\\prop_fake_ladder";
  pub const ENGINE_SHADER: &'static str = "def_shaders\\def_trans_v";
  pub const COMPILER_SHADER: &'static str = "def_shaders\\def_kolizion_vertex";

  /// The game material of a ladder whose packet keeps none (before version 127): every vanilla ladder's but the
  /// wooden ones.
  pub const DEFAULT_GAME_MATERIAL: &'static str = "materials\\fake_ladders";

  /// The half thickness below which a box is flat and only its front comes back. The Build's `f32` fit leaves a flat
  /// ladder a few micrometres (vanilla's thinnest 1.9 µm), and side faces that thin give normals too rough for
  /// `OrientToNorm`.
  pub const FLAT_HALF_THICKNESS: f32 = 1e-4;

  /// The entity's ladder as `levels\<level>\<level>_ladder_<index>`, its pivot at the base of the box, or `None` for
  /// an entity that is no ladder or whose shape is not one box.
  ///
  /// # Errors
  ///
  /// Returns an error when the decompile refuses the box's faces.
  pub fn decompile(name: &str, index: usize, entity: &AlifeObject) -> XrfResult<Option<LevelDecompiledLadder>> {
    let AlifeObjectInherited::CseAlifeObjectClimable(climable) = &entity.inherited else {
      return Ok(None);
    };
    let [Shape::Box(shape)] = climable.base.shape.as_slice() else {
      return Ok(None);
    };
    let half: [f32; 3] = [shape.x_axis.length(), shape.y_axis.length(), shape.z_axis.length()];
    let frame: Affine3d = Affine3d::from_xyz(&entity.header.direction);
    let base: Vector3d = Self::get_base(&half, &frame);
    let triangles: Vec<[Vector3d; 3]> = Self::get_faces(&half)
      .into_iter()
      .map(|triangle| triangle.map(|point| &point - &base))
      .collect();
    let is_material_inferred: bool = climable.game_material.is_empty();
    let dress: SurfaceDress = SurfaceDress {
      compiler_shader: String::from(Self::COMPILER_SHADER),
      game_material: if is_material_inferred {
        String::from(Self::DEFAULT_GAME_MATERIAL)
      } else {
        climable.game_material.clone()
      },
    };

    Ok(Some(LevelDecompiledLadder {
      object: LevelDecompiledObject {
        reference: format!("levels\\{name}\\{name}_ladder_{index}"),
        transform: EditableObjectTransform {
          position: &frame.transform_direction(&base) + &entity.header.position,
          // `setXYZ` of the packet's angles, which a scene object's rotation composes too.
          rotation: entity.header.direction,
          scale: Vector3d::ONE,
        },
        source: String::from(LevelEntityConversion::SPAWN_FILE_NAME),
        decompile: ObjectDecompiler::decompile_level_geometry(&LevelVisualGeometry::of_triangles(
          Self::TEXTURE,
          Self::ENGINE_SHADER,
          &dress,
          &triangles,
        ))?,
      },
      is_material_inferred,
    }))
  }

  /// The faces, wound outward, of the box of half sizes `half` less the face behind its thinnest axis; its front alone
  /// where it is flat ([`Self::FLAT_HALF_THICKNESS`]).
  fn get_faces(half: &[f32; 3]) -> Vec<[Vector3d; 3]> {
    let thin: usize = Self::get_thin_axis(half);
    let corner = |signs: [f32; 3]| Vector3d::new(signs[0] * half[0], signs[1] * half[1], signs[2] * half[2]);
    let mut faces: Vec<[Vector3d; 3]> = Vec::with_capacity(10);

    for axis in 0..3 {
      for sign in [1.0_f32, -1.0] {
        let is_front: bool = axis == thin && sign > 0.0;
        let is_back: bool = axis == thin && sign < 0.0;

        if is_back || (!is_front && half[thin] < Self::FLAT_HALF_THICKNESS) {
          continue;
        }

        // The face's corners in turn about its normal: along the next axis, then the one after.
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        let at = |a: f32, b: f32| {
          let mut signs: [f32; 3] = [0.0; 3];

          signs[axis] = sign;
          signs[u] = a;
          signs[v] = b * sign;
          corner(signs)
        };
        let quad: [Vector3d; 4] = [at(-1.0, -1.0), at(1.0, -1.0), at(1.0, 1.0), at(-1.0, 1.0)];

        faces.push([quad[0], quad[1], quad[2]]);
        faces.push([quad[0], quad[2], quad[3]]);
      }
    }

    faces
  }

  /// The axis the box is thinnest along, which the export turned to face the way the faces did.
  fn get_thin_axis(half: &[f32; 3]) -> usize {
    (0..3)
      .min_by(|first, second| half[*first].total_cmp(&half[*second]))
      .unwrap_or(0)
  }

  /// The middle of the box's lower end, where its scene object stands, along whichever other axis rises most in the
  /// level.
  fn get_base(half: &[f32; 3], frame: &Affine3d) -> Vector3d {
    let thin: usize = Self::get_thin_axis(half);
    let axes: [Vector3d; 3] = [Vector3d::X, Vector3d::Y, Vector3d::Z];
    let (axis, rise) = (0..3)
      .filter(|axis| *axis != thin)
      .map(|axis| (axis, frame.transform_direction(&axes[axis]).y))
      .max_by(|first, second| first.1.abs().total_cmp(&second.1.abs()))
      .unwrap_or((1, 1.0));

    &axes[axis] * (-rise.signum() * half[axis])
  }
}
