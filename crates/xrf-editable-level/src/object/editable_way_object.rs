use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};
use xrf_math::{Vector2d, Vector3d};

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// A link from one way point to another of its way, `SWPLink`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditableWayLink {
  /// The point it leads to, by its place in the way's points.
  pub to: usize,
  pub probability: f32,
}

/// One point of a way, `CWayPoint`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableWayPoint {
  /// Unique within its way, `wp00` upwards for a point the editor adds.
  pub name: String,
  pub position: Vector3d,
  /// Thirty-two flags the scripts give meaning to.
  pub flags: u32,
  pub selected: bool,
  /// Its links in the order the editor keeps them.
  pub links: Vec<EditableWayLink>,
}

/// A patrol path, `CWayObject`: an object of `way.part`, its points and links. The build exports every way into
/// `level.game`'s patrol path table under the object's name.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableWayObject {
  pub object: EditableCustomObject,
  pub points: Vec<EditableWayPoint>,
  /// `EWayType`, [`Self::PATROL_PATH`] for every way the editor makes.
  pub kind: u32,
}

impl EditableWayObject {
  /// `WAYOBJECT_VERSION`, the only one `LoadLTX` reads.
  pub const VERSION: u32 = 0x13;

  /// `wtPatrolPath`.
  pub const PATROL_PATH: u32 = 0;

  const LINK_PREFIX: &'static str = "link_wp_";

  /// A link's key, `link_wp_%4d_%4d`: the padding sorts the keys in the order of their numbers.
  fn get_link_key(point: usize, link: usize) -> String {
    format!("{}{point:4}_{link:4}", Self::LINK_PREFIX)
  }

  /// A link key's point and link numbers, as `sscanf("link_wp_%4d_%4d")` reads them.
  fn parse_link_key(key: &str) -> Option<(usize, usize)> {
    let (point, link) = key.strip_prefix(Self::LINK_PREFIX)?.split_once('_')?;

    Some((point.trim().parse().ok()?, link.trim().parse().ok()?))
  }
}

impl EditableToolObject for EditableWayObject {
  /// `OBJCLASS_WAY`.
  const CLASS_ID: i32 = 7;
  const TOOL_NAME: &'static str = "way";

  /// As `CWayObject::LoadLTX` reads it: another version or an empty name refused, the points, then each link in the
  /// order its key sorts, appended to its point.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    let section: &IniSection = object.section;
    let version: u32 = section.get_parsed::<u32>("version")?;
    let custom: EditableCustomObject = EditableCustomObject::read_from_section(section)?;

    if version != Self::VERSION || custom.name.is_empty() {
      return Err(XrfError::new_parsing_error(format!(
        "Way [{}] is of version {version} named '{}', not a named way of version {}",
        section.name,
        custom.name,
        Self::VERSION
      )));
    }

    let count: u32 = EditableLevelValue::read_u32(section, "wp_count")?;
    let mut points: Vec<EditableWayPoint> = (0..count)
      .map(|index| {
        Ok(EditableWayPoint {
          name: section.get_required(&format!("wp_{index}_name"))?.to_owned(),
          position: EditableLevelValue::read_vector3(section, &format!("wp_{index}_pos"))?,
          flags: EditableLevelValue::read_u32(section, &format!("wp_{index}_flags"))?,
          selected: EditableLevelValue::read_bool(section, &format!("wp_{index}_selected"))?,
          links: Vec::new(),
        })
      })
      .collect::<XrfResult<Vec<EditableWayPoint>>>()?;
    let mut keys: Vec<&str> = section
      .fields
      .iter()
      .map(|field| field.key.as_str())
      .filter(|key| key.starts_with(Self::LINK_PREFIX))
      .collect();

    keys.sort_unstable();

    for key in keys {
      let link_error = || XrfError::new_parsing_error(format!("Way [{}] link {key} is malformed", section.name));
      let (point, _) = Self::parse_link_key(key).ok_or_else(link_error)?;
      let target: Vector2d = EditableLevelValue::read_vector2(section, key)?;
      let to: usize = usize::try_from(target.x.floor() as i64).map_err(|_| link_error())?;

      if point >= points.len() || to >= points.len() {
        return Err(link_error());
      }

      points[point].links.push(EditableWayLink {
        to,
        probability: target.y,
      });
    }

    Ok(Self {
      object: custom,
      points,
      kind: EditableLevelValue::read_u32(section, "type")?,
    })
  }

  /// As `CWayObject::SaveLTX` writes it, at [`Self::VERSION`].
  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    let section: &mut IniSection = ini.section_mut(section);

    self.object.write_to_section(section);
    section.set("version", Self::VERSION.to_string());
    section.set("wp_count", self.points.len().to_string());

    for (index, point) in self.points.iter().enumerate() {
      section.set(
        format!("wp_{index}_pos"),
        EditableLevelValue::format_vector3(&point.position),
      );
      section.set(format!("wp_{index}_flags"), EditableLevelValue::format_u32(point.flags));
      section.set(
        format!("wp_{index}_selected"),
        EditableLevelValue::format_bool(point.selected),
      );
      section.set(format!("wp_{index}_name"), point.name.clone());
    }

    for (index, point) in self.points.iter().enumerate() {
      for (link_index, link) in point.links.iter().enumerate() {
        section.set(
          Self::get_link_key(index, link_index),
          EditableLevelValue::format_vector2(&Vector2d::new(link.to as f32, link.probability)),
        );
      }
    }

    section.set("type", EditableLevelValue::format_u32(self.kind));
  }
}
