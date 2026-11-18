use std::collections::HashMap;

use xrf_editable_level::EditableSectorObject;
use xrf_math::{Aabb, Sphere, Vector3d};

use crate::level::build::level_build_visibility::LevelBuildVisibility;

/// A scene's sectors as the Build numbers and tests them (`SceneBuilder::RenumerateSectors`), in the scene's order.
///
/// The default sector is the one marked so, else one past the last. Each sector's volume is the box of its items'
/// placed mesh boxes (`CSector::UpdateVolume`).
pub(crate) struct LevelBuildSectors {
  numbers: HashMap<String, u16>,
  /// Each item, object then mesh name, by the sector naming it first.
  items: HashMap<(String, String), u16>,
  volumes: Vec<Option<(Aabb, Sphere)>>,
  default: u16,
}

impl LevelBuildSectors {
  /// The sectors of `sectors`, `place` giving each item's placed mesh box, or `None` to add nothing to the volume.
  pub(crate) fn new(sectors: &[EditableSectorObject], place: impl Fn(&str, &str) -> Option<Aabb>) -> Self {
    let mut numbers: HashMap<String, u16> = HashMap::with_capacity(sectors.len());
    let mut items: HashMap<(String, String), u16> = HashMap::new();
    let mut volumes: Vec<Option<(Aabb, Sphere)>> = Vec::with_capacity(sectors.len());
    let mut default: Option<u16> = None;

    for (sector, number) in sectors.iter().zip(0u16..) {
      numbers.insert(sector.object.name.clone(), number);

      if sector.is_default {
        default = Some(number);
      }

      let mut volume: Option<Aabb> = None;

      for item in &sector.items {
        items.entry((item.object.clone(), item.mesh.clone())).or_insert(number);

        if let Some(placed) = place(&item.object, &item.mesh) {
          match &mut volume {
            Some(volume) => {
              volume.include_point(&placed.min);
              volume.include_point(&placed.max);
            }
            None => volume = Some(placed),
          }
        }
      }

      volumes.push(volume.map(|volume| (volume, volume.get_bounding_sphere())));
    }

    Self {
      numbers,
      items,
      volumes,
      default: default.unwrap_or_else(|| u16::try_from(sectors.len()).unwrap_or(u16::MAX)),
    }
  }

  pub(crate) fn get_number(&self, name: &str) -> Option<u16> {
    self.numbers.get(name).copied()
  }

  /// The sector of a placed object's mesh, `PortalUtils.FindSector`: the first naming it, else the default.
  pub(crate) fn find(&self, object: &str, mesh: &str) -> u16 {
    self
      .items
      .get(&(object.to_owned(), mesh.to_owned()))
      .copied()
      .unwrap_or(self.default)
  }

  /// The sector a sphere lies in, `SceneBuilder::CalculateSector`: the first other than the default it lies inside,
  /// partly or fully, else the default.
  pub(crate) fn calculate(&self, center: &Vector3d, radius: f32) -> u16 {
    (0..self.volumes.len())
      .filter_map(|index| u16::try_from(index).ok())
      .find(|number| {
        *number != self.default
          && matches!(
            self.intersect(*number, center, radius),
            LevelBuildVisibility::PartialInside | LevelBuildVisibility::Fully
          )
      })
      .unwrap_or(self.default)
  }

  /// The sectors a dynamic light of range `radius` at `center` reaches, as `SceneBuilder::BuildLight` finds them:
  /// those it lies inside, then those it reaches from outside, at most 16. The default alone for a scene of no sector;
  /// empty where it reaches none, which the Build refuses.
  pub(crate) fn get_light_sectors(&self, center: &Vector3d, radius: f32) -> Vec<u16> {
    if self.volumes.is_empty() {
      return vec![self.default];
    }

    let numbers = || (0..self.volumes.len()).filter_map(|index| u16::try_from(index).ok());
    let inside = numbers().filter(|number| {
      matches!(
        self.intersect(*number, center, radius),
        LevelBuildVisibility::PartialInside | LevelBuildVisibility::Fully
      )
    });
    let outside =
      numbers().filter(|number| self.intersect(*number, center, radius) == LevelBuildVisibility::PartialOutside);

    inside.chain(outside).take(Self::LIGHT_SECTORS).collect()
  }

  /// `svector<u16, 16>`'s capacity.
  const LIGHT_SECTORS: usize = 16;

  /// `CSector::Intersect(center, radius)`.
  fn intersect(&self, number: u16, center: &Vector3d, radius: f32) -> LevelBuildVisibility {
    let Some(Some((volume, sphere))) = self.volumes.get(usize::from(number)) else {
      return LevelBuildVisibility::None;
    };
    let distance: f32 = sphere.center.distance(center);
    let half: Vector3d = &volume.get_size() * 0.5;
    let is_in_sphere: bool = distance + radius < sphere.radius && radius > half.x && radius > half.y && radius > half.z;
    let is_contained: bool = (volume.min.x..=volume.max.x).contains(&center.x)
      && (volume.min.y..=volume.max.y).contains(&center.y)
      && (volume.min.z..=volume.max.z).contains(&center.z);

    match (is_contained, is_in_sphere) {
      (true, true) => LevelBuildVisibility::Fully,
      (true, false) => LevelBuildVisibility::PartialInside,
      (false, _) if distance < radius + sphere.radius => LevelBuildVisibility::PartialOutside,
      (false, _) => LevelBuildVisibility::None,
    }
  }
}
