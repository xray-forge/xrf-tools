use xrf_error::{XrfError, XrfResult};
use xrf_level::{DetailVertex, LevelVertex};
use xrf_math::{Vector2d, Vector3d};
use xrf_ogf::{OgfStaticVertex, OgfVertex};

use crate::object::decompile::geometry::source::bone_influences::BoneInfluences;
use crate::object::decompile::geometry::source::normal_precision::NormalPrecision;

/// One corner of a compiled triangle as an OGF, a level or a detail model stores it: where it stands, the normal the
/// export gave it and how precisely that is stored (zero where none is), its texture coordinate and the bones it
/// follows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OgfCorner {
  pub(crate) position: Vector3d,
  pub(crate) normal: Vector3d,
  pub(crate) normal_precision: NormalPrecision,
  pub(crate) texture_coordinate: Vector2d,
  pub(crate) influences: BoneInfluences,
}

impl OgfCorner {
  /// Whether the two corners carry one smoothed normal; where either stores none, nothing tells them apart, so they do.
  pub(crate) fn has_normal_of(&self, other: &Self) -> bool {
    self.is_near(&self.normal, &other.normal, other).unwrap_or(true)
  }

  /// Whether this corner's normal is the other's turned around, as a two-sided copy's are; where either stores none,
  /// nothing shows it is.
  pub(crate) fn has_normal_opposite(&self, other: &Self) -> bool {
    self.is_near(&self.normal, &-&other.normal, other).unwrap_or(false)
  }

  /// The corner halfway to `other`: the middle of their positions and texture coordinates, this corner's normal where
  /// both carry one smoothed normal, else the middle of the two made unit, and this corner's bones.
  pub(crate) fn interpolate(&self, other: &Self) -> Self {
    let normal: Vector3d = if self.has_normal_of(other) {
      self.normal
    } else {
      let sum: Vector3d = &self.normal + &other.normal;
      let length: f32 = sum.length();

      if length.is_normal() { &sum / length } else { self.normal }
    };

    Self {
      position: &(&self.position + &other.position) * 0.5,
      normal,
      normal_precision: self.normal_precision,
      texture_coordinate: Vector2d::new(
        f32::midpoint(self.texture_coordinate.x, other.texture_coordinate.x),
        f32::midpoint(self.texture_coordinate.y, other.texture_coordinate.y),
      ),
      influences: self.influences,
    }
  }

  /// Whether the corner stores a normal, which a detail model's corners do not.
  pub(crate) fn has_normal(&self) -> bool {
    self.normal_precision != NormalPrecision::Unstored
  }

  /// The position's bits and the bones followed, which narrow a welded point to those standing there: the export copied
  /// one float into every corner split from a point. The weights stay out of the key, since a point exported into
  /// children of different link layouts comes back with weights rounded differently; [`BoneInfluences::is_alike`] tells
  /// the points of one key apart.
  pub(crate) fn get_weld_key(&self) -> [u32; 3 + BoneInfluences::CAPACITY] {
    let bones: [u32; BoneInfluences::CAPACITY] = self.influences.get_bone_key();

    [
      self.position.x.to_bits(),
      self.position.y.to_bits(),
      self.position.z.to_bits(),
      bones[0],
      bones[1],
      bones[2],
      bones[3],
    ]
  }

  /// Whether `first` and `second` are one normal within the coarser precision of this corner and `other`, or `None`
  /// where either stores no normal.
  fn is_near(&self, first: &Vector3d, second: &Vector3d, other: &Self) -> Option<bool> {
    let tolerance: f32 = self.normal_precision.get_tolerance_with(other.normal_precision)?;

    Some(
      (first.x - second.x).abs() <= tolerance
        && (first.y - second.y).abs() <= tolerance
        && (first.z - second.z).abs() <= tolerance,
    )
  }
}

/// A skinned vertex's corner, with the bones its links name.
impl From<&OgfVertex> for OgfCorner {
  fn from(vertex: &OgfVertex) -> Self {
    Self {
      position: vertex.position,
      normal: vertex.normal,
      normal_precision: NormalPrecision::Float,
      texture_coordinate: Vector2d::new(vertex.texture_u, vertex.texture_v),
      influences: BoneInfluences::of_links(&vertex.links),
    }
  }
}

/// A static vertex's corner, from its normal and first texture coordinate set: the export writes position, normal and
/// one coordinate pair (`D3DFVF_XYZ | D3DFVF_NORMAL | D3DFVF_TEX1`).
impl TryFrom<&OgfStaticVertex> for OgfCorner {
  type Error = XrfError;

  /// # Errors
  ///
  /// Returns an error for a layout without a normal or a two-component first coordinate set, which no editable object
  /// was exported to.
  fn try_from(vertex: &OgfStaticVertex) -> XrfResult<Self> {
    let normal: Vector3d = vertex
      .normal
      .ok_or_else(|| XrfError::new_not_implemented_error("A static OGF child whose vertices store no normal"))?;
    let texture_coordinate: Vector2d = match vertex.texture_coordinates.first().map(Vec::as_slice) {
      Some([u, v]) => Vector2d::new(*u, *v),
      _ => {
        return Err(XrfError::new_not_implemented_error(
          "A static OGF child whose vertices store no two-component texture coordinate",
        ));
      }
    };

    Ok(Self {
      position: vertex.position,
      normal,
      normal_precision: NormalPrecision::Float,
      texture_coordinate,
      influences: BoneInfluences::NONE,
    })
  }
}

/// A level vertex's corner, its normal stored a byte a component in `level.geom`.
impl From<&LevelVertex> for OgfCorner {
  fn from(vertex: &LevelVertex) -> Self {
    Self {
      position: vertex.position,
      normal: vertex.normal,
      normal_precision: NormalPrecision::Byte,
      texture_coordinate: vertex.texture_coordinate,
      influences: BoneInfluences::NONE,
    }
  }
}

/// A detail model vertex's corner, which stores a position and a texture coordinate and no normal.
impl From<&DetailVertex> for OgfCorner {
  fn from(vertex: &DetailVertex) -> Self {
    Self {
      position: vertex.position,
      normal: Vector3d::ZERO,
      normal_precision: NormalPrecision::Unstored,
      texture_coordinate: Vector2d::new(vertex.u, vertex.v),
      influences: BoneInfluences::NONE,
    }
  }
}
