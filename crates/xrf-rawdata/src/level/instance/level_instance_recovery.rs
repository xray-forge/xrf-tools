use std::collections::BTreeMap;

use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::level::instance::level_instance_library::LevelInstanceLibrary;
use crate::level::instance::level_instance_matcher::LevelInstanceMatcher;
use crate::level::instance::level_instance_placement::LevelInstancePlacement;
use crate::level::instance::level_instance_triangles::LevelInstanceTriangles;
use crate::level::level_collision_index::LevelCollisionIndex;
use crate::level::level_visual_geometry::LevelVisualGeometry;

/// The library objects found placed in a level's static geometry, sector by sector ([`LevelInstanceMatcher`]): each
/// placement, and the triangles it takes out of each visual.
pub(crate) struct LevelInstanceRecovery {
  pub(crate) placements: Vec<LevelInstancePlacement>,
  /// For each sector given and each of its visuals, the triangles the placements took, by their place in the visual,
  /// sorted.
  pub(crate) taken: Vec<Vec<Vec<u32>>>,
}

impl LevelInstanceRecovery {
  /// The placements of `library`'s objects in `sectors`' static visuals, each sector's by its compiled index; a
  /// placement belongs to the sector holding most of its triangles. Where `collision` is given, each placement's
  /// collision-only triangles cover their faces there, so they come back with it rather than as collision of their own.
  ///
  /// # Errors
  ///
  /// Returns an error when a placement's fit is no transform the editor saves.
  pub(crate) fn find(
    library: &LevelInstanceLibrary,
    sectors: &[&[LevelVisualGeometry]],
    mut collision: Option<&mut LevelCollisionIndex>,
  ) -> XrfResult<Self> {
    // Each visual indexed, by its sector and its place there.
    let owners: Vec<(usize, usize)> = sectors
      .iter()
      .enumerate()
      .flat_map(|(sector, visuals)| (0..visuals.len()).map(move |visual| (sector, visual)))
      .collect();
    let visuals: Vec<&LevelVisualGeometry> = sectors.iter().flat_map(|visuals| visuals.iter()).collect();
    let mut index: LevelInstanceTriangles = LevelInstanceTriangles::new(library, &visuals);
    let matches = LevelInstanceMatcher::find(&library.objects, &mut index, LevelInstanceMatcher::COVERAGE);
    let mut taken: Vec<Vec<Vec<u32>>> = sectors.iter().map(|visuals| vec![Vec::new(); visuals.len()]).collect();
    let mut placements: Vec<LevelInstancePlacement> = Vec::with_capacity(matches.len());

    for found in &matches {
      let mut held: BTreeMap<usize, usize> = BTreeMap::new();

      for triangle in &found.claimed {
        let triangle = &index.triangles[*triangle as usize];
        let (sector, visual) = owners[triangle.visual as usize];

        *held.entry(sector).or_default() += 1;
        taken[sector][visual].push(triangle.index);
      }

      let object = &library.objects[found.object];

      if let Some(collision) = collision.as_deref_mut() {
        for points in &object.collision {
          collision.cover_instance(&points.map(|point| {
            let placed = found.fit.apply(point);

            Vector3d::new(placed.x as f32, placed.y as f32, placed.z as f32)
          }));
        }
      }

      let sector: usize = held
        .into_iter()
        .max_by_key(|(sector, count)| (*count, std::cmp::Reverse(*sector)))
        .map_or(0, |(sector, _)| sector);

      placements.push(LevelInstancePlacement {
        reference: object.reference.clone(),
        path: object.path.clone(),
        object: String::new(),
        meshes: object.meshes.clone(),
        transform: found.fit.to_transform()?,
        sector: u16::try_from(sector).unwrap_or(u16::MAX),
        coverage: found.coverage,
        error: found.error,
        triangles: found.claimed.len(),
      });
    }

    for triangles in taken.iter_mut().flatten() {
      triangles.sort_unstable();
      triangles.dedup();
    }

    Ok(Self { placements, taken })
  }
}
