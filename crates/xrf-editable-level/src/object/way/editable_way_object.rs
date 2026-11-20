use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};
use xrf_math::Vector2d;

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::object::way::editable_way_link::EditableWayLink;
use crate::object::way::editable_way_point::EditableWayPoint;
use crate::part::editable_part_object::EditablePartObject;

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

  /// The most digits `%4d` reads of a number; a key holding more fails the editor's `R_ASSERT4(res == 2, …)`.
  const LINK_KEY_DIGITS: usize = 4;

  /// A link's key, `link_wp_%4d_%4d`: the padding sorts the keys in the order of their numbers.
  fn get_link_key(point: usize, link: usize) -> String {
    format!("{}{point:4}_{link:4}", Self::LINK_PREFIX)
  }

  /// A link key's point and link numbers, as `sscanf("link_wp_%4d_%4d")` reads them: each at most four digits after
  /// the padding.
  fn parse_link_key(key: &str) -> Option<(usize, usize)> {
    let (point, link) = key.strip_prefix(Self::LINK_PREFIX)?.split_once('_')?;
    let number = |field: &str| -> Option<usize> {
      let digits: &str = field.trim_start();

      (!digits.is_empty() && digits.len() <= Self::LINK_KEY_DIGITS && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| digits.parse().ok())
        .flatten()
    };

    Some((number(point)?, number(link)?))
  }

  /// Whether every link leads to another point of the way: `CWayPoint::CreateLink` drops a link to its own point, a link
  /// past the points names none, and a link from or to a point past 9999 has a key `%4d` cannot read back.
  ///
  /// # Errors
  ///
  /// Returns an invalid error naming the way, the point and the link.
  pub fn check_links(&self) -> XrfResult {
    // The first point number a link key's four digits cannot spell.
    const UNKEYED: usize = 10_000;

    for (index, point) in self.points.iter().enumerate() {
      for link in &point.links {
        if index.max(link.to) >= UNKEYED {
          return Err(XrfError::new_invalid_error(format!(
            "Way '{}' point {index} links to point {}, past the {UNKEYED} points a link key's {} digits number",
            self.object.name,
            link.to,
            Self::LINK_KEY_DIGITS
          )));
        }

        if link.to == index {
          return Err(XrfError::new_invalid_error(format!(
            "Way '{}' point {index} links to itself, a link the editor drops",
            self.object.name
          )));
        }

        if link.to >= self.points.len() {
          return Err(XrfError::new_invalid_error(format!(
            "Way '{}' point {index} links to point {} of {}",
            self.object.name,
            link.to,
            self.points.len()
          )));
        }
      }
    }

    Ok(())
  }
}

impl EditableToolObject for EditableWayObject {
  /// `OBJCLASS_WAY`.
  const CLASS_ID: i32 = 7;
  const TOOL_NAME: &'static str = "way";

  /// As `CWayObject::LoadLTX` reads it, each link appended to its point in key order. Refuses another version, an empty
  /// name, a link key the editor's `sscanf` cannot read, and a link [`Self::check_links`] refuses.
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
      .fields()
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

      points
        .get_mut(point)
        .ok_or_else(link_error)?
        .links
        .push(EditableWayLink {
          to,
          probability: target.y,
        });
    }

    let way: Self = Self {
      object: custom,
      points,
      kind: EditableLevelValue::read_u32(section, "type")?,
    };

    way.check_links()?;

    Ok(way)
  }

  /// As `CWayObject::SaveLTX` writes it, at [`Self::VERSION`]. Links are written unchecked; every reader and the
  /// level's export run [`Self::check_links`].
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
