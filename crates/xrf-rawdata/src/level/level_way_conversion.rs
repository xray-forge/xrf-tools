use xrf_editable_level::{
  EditableCustomObject, EditableObjectPart, EditableObjectTransform, EditableWayLink, EditableWayObject,
  EditableWayPoint,
};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelGameWay, LevelGameWayLink, LevelGameWayPoint};

use crate::level::level_decompile_context::LevelDecompileContext;

/// A level's patrol paths, between the scene's `way.part` and the patrol path table of `level.game`, which
/// `CWayObject::ExportGame` (`WayPoint.cpp:854`) fills. The file's respawn point table comes from the spawn tool.
pub struct LevelWayConversion;

impl LevelWayConversion {
  /// The way part the patrol paths were exported from: a way per path under its name, standing where a new object
  /// does, of the type a path keeps (none ships one, so a patrol path), each point unselected, and each link appended to
  /// the point it leaves, in the file's order.
  ///
  /// # Errors
  ///
  /// Returns an error naming a path whose link names a point it does not hold or its own point, which the editor never
  /// exports.
  pub fn decompile(
    ways: &[LevelGameWay],
    context: &mut LevelDecompileContext,
  ) -> XrfResult<EditableObjectPart<EditableWayObject>> {
    let objects: Vec<EditableWayObject> = ways
      .iter()
      .map(|way| {
        let mut points: Vec<EditableWayPoint> = way
          .points
          .iter()
          .map(|point| EditableWayPoint {
            name: point.name.clone(),
            position: point.position,
            flags: point.flags,
            selected: false,
            links: Vec::new(),
          })
          .collect();

        for link in &way.links {
          let (from, to) = (usize::from(link.from), usize::from(link.to));
          let point: &mut EditableWayPoint = points.get_mut(from).ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "Patrol path '{}' links from point {from} of {}",
              way.name,
              way.points.len()
            ))
          })?;

          point.links.push(EditableWayLink {
            to,
            probability: link.probability,
          });
        }

        let object: EditableWayObject = EditableWayObject {
          object: EditableCustomObject::new(way.name.clone(), EditableObjectTransform::default()),
          points,
          kind: way.kind.unwrap_or(EditableWayObject::PATROL_PATH),
        };

        object.check_links()?;

        Ok(object)
      })
      .collect::<XrfResult<Vec<EditableWayObject>>>()?;

    Ok(EditableObjectPart::new(context.guid, objects))
  }

  /// The patrol paths `ExportGame` writes from the part, one per way in the part's order: `WAYOBJECT_VERSION`, the
  /// way's name, its points' positions, flags and names, and their links point by point; no type chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when a way's link leads to its own point or past the way's points
  /// ([`EditableWayObject::check_links`]), or a way's point is past what the file's `u16` indices name.
  pub fn export(part: &EditableObjectPart<EditableWayObject>) -> XrfResult<Vec<LevelGameWay>> {
    part
      .objects
      .iter()
      .map(|way| {
        way.check_links()?;

        let mut links: Vec<LevelGameWayLink> = Vec::new();

        let index = |index: usize| {
          u16::try_from(index).map_err(|_| {
            XrfError::new_invalid_error(format!(
              "Way '{}' point {index} is past what a patrol path counts",
              way.object.name
            ))
          })
        };

        for (from, point) in way.points.iter().enumerate() {
          for link in &point.links {
            links.push(LevelGameWayLink {
              from: index(from)?,
              to: index(link.to)?,
              probability: link.probability,
            });
          }
        }

        Ok(LevelGameWay {
          version: EditableWayObject::VERSION as u16,
          name: way.object.name.clone(),
          kind: None,
          points: way
            .points
            .iter()
            .map(|point| LevelGameWayPoint {
              position: point.position,
              flags: point.flags,
              name: point.name.clone(),
            })
            .collect(),
          links,
        })
      })
      .collect()
  }
}
