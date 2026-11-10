use std::collections::BTreeMap;

use xrf_editable_level::{
  EditableCustomObject, EditableLevelOptions, EditableLevelValue, EditableObjectTransform, EditableSectorObject,
  EditableShape, EditableShapeElement, EditableShapeObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, Affine3d, Vector3d};

use crate::level::level_decompile_context::LevelDecompileContext;

/// `level.ltx`, the map's settings the Build writes from the scene (`SceneBuilder::BuildLTX`, `BuilderLTX.cpp:25`), and
/// back: the map's version and the game types it serves, the map's rectangle and texture, the build options' text, and
/// each sector's part of the map.
pub struct LevelMapConversion;

/// What `level.ltx` says of the scene that built it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelMapSettings {
  /// `[map_usage] ver`.
  pub map_version: Option<String>,
  /// The game types `[map_usage]` names, by their `EGameIDs` bits; single player where it names none.
  pub game_types: u16,
  /// `[level_map] bound_rect`: the lowest x and z, then the highest.
  pub bounds: Option<[f32; 4]>,
  /// `[level_map] texture`.
  pub texture: Option<String>,
  /// `[sub_level_map]`: each sector's part of the map, by the sector's number.
  pub sector_maps: BTreeMap<u16, u8>,
  /// Sections other than those the Build writes itself, which came from the build options' text or a later edit.
  pub other_sections: Vec<String>,
  /// Names `[map_usage]` holds that are no game type the Build writes.
  pub unknown_game_types: Vec<String>,
}

impl LevelMapConversion {
  /// The file the build writes the map's settings to.
  pub const FILE_NAME: &'static str = "level.ltx";

  /// The name a level-bound shape takes.
  pub const BOUND_PREFIX: &'static str = "level_bound";

  /// Each game type `[map_usage]` names, by its `EGameIDs` bit (`game_base_space.h`), as `BuildLTX` writes them.
  const GAME_TYPES: [(u16, &'static str); 6] = [
    (1 << 1, "deathmatch"),
    (1 << 2, "teamdeathmatch"),
    (1 << 3, "artefacthunt"),
    (1 << 4, "capturetheartefact"),
    (1 << 5, "dominationzone"),
    (1 << 6, "teamdominationzone"),
  ];

  /// The sections the Build writes itself.
  const BUILT_SECTIONS: [&'static str; 3] = ["map_usage", "level_map", "sub_level_map"];

  /// How tall the level-bound shape stands, which `level.ltx` does not keep.
  const BOUND_HEIGHT: f32 = 1.0;

  /// The settings `level.ltx` holds, read line by line as the Build writes them and a later save of the file leaves
  /// them: sections, `key = value` lines and bare names, `;` lines and blank ones skipped, a line read up to a zero
  /// byte (which ends the build options' text).
  ///
  /// # Errors
  ///
  /// Returns an error for a `bound_rect` that is not four floats, or a `[sub_level_map]` line that is not a sector's
  /// number and a map's part.
  pub fn decompile(bytes: &[u8]) -> XrfResult<LevelMapSettings> {
    let text: String = bytes.iter().map(|byte| char::from(*byte)).collect();
    let mut settings: LevelMapSettings = LevelMapSettings::default();
    let mut section: String = String::new();

    for line in text.lines() {
      let line: &str = line.split('\0').next().unwrap_or_default().trim();

      if line.is_empty() || line.starts_with(';') {
        continue;
      }

      if let Some(name) = line.strip_prefix('[').and_then(|line| line.strip_suffix(']')) {
        section = name.to_owned();

        if !Self::BUILT_SECTIONS.contains(&name) && !settings.other_sections.contains(&section) {
          settings.other_sections.push(section.clone());
        }

        continue;
      }

      let (key, value) = match line.split_once('=') {
        Some((key, value)) => (key.trim(), Some(value.trim())),
        None => (line, None),
      };

      match (section.as_str(), key, value) {
        ("map_usage", "ver", Some(value)) => settings.map_version = Some(value.to_owned()),
        ("map_usage", name, None) => match Self::GAME_TYPES.iter().find(|(_, known)| *known == name) {
          Some((bit, _)) => settings.game_types |= bit,
          None => settings.unknown_game_types.push(name.to_owned()),
        },
        ("level_map", "bound_rect", Some(value)) => settings.bounds = Some(Self::read_bounds(value)?),
        ("level_map", "texture", Some(value)) => settings.texture = Some(value.to_owned()),
        ("sub_level_map", number, Some(map)) => {
          let parsed = number.parse::<u16>().ok().zip(map.parse::<u8>().ok());
          let (number, map) = parsed.ok_or_else(|| {
            XrfError::new_parsing_error(format!(
              "[sub_level_map] '{line}' is not a sector's number and its part of the map"
            ))
          })?;

          settings.sector_maps.insert(number, map);
        }
        _ => {}
      }
    }

    if settings.game_types == 0 {
      settings.game_types = EditableLevelOptions::SINGLE_PLAYER;
    }

    Ok(settings)
  }

  /// The level-bound shape whose box the Build takes for a map of `bounds` (lowest x and z, then highest): a box over
  /// them, [`Self::BOUND_HEIGHT`] tall around height 0, its centre in the element and the object at the origin, so the
  /// object's transform moves no corner off the box's ends.
  pub fn decompile_bound(bounds: &[f32; 4], context: &mut LevelDecompileContext) -> EditableShapeObject {
    let [min_x, min_z, max_x, max_z] = *bounds;
    let (centre_x, width) = Self::fit_span(min_x, max_x);
    let (centre_z, depth) = Self::fit_span(min_z, max_z);

    EditableShapeObject {
      object: EditableCustomObject::new(
        context.names.get_next(Self::BOUND_PREFIX),
        EditableObjectTransform::default(),
      ),
      shape: EditableShape {
        usage: EditableShape::LEVEL_BOUND_USAGE,
        elements: vec![EditableShapeElement::Box(Affine3d {
          x_axis: Vector3d::new(width, 0.0, 0.0),
          y_axis: Vector3d::new(0.0, Self::BOUND_HEIGHT, 0.0),
          z_axis: Vector3d::new(0.0, 0.0, depth),
          translation: Vector3d::new(centre_x, 0.0, centre_z),
        })],
      },
    }
  }

  /// A centre and a width whose unit box ends where `min` and `max` are, `-0.5` and `0.5` times the width added to the
  /// centre in single precision as the editor places a box's corners: the plain middle and span, or the nearest of a
  /// few single-precision steps around them that ends exactly there, since the plain ones can miss by a step and
  /// `level.ltx`'s six decimals show it. The plain ones where no nearby pair ends there.
  fn fit_span(min: f32, max: f32) -> (f32, f32) {
    let (centre, width) = (min + (max - min) / 2.0, max - min);
    let steps = |value: f32| {
      let (mut down, mut up) = (value, value);

      std::iter::once(value).chain((0..Self::FIT_STEPS).flat_map(move |_| {
        down = down.next_down();
        up = up.next_up();
        [down, up]
      }))
    };

    steps(centre)
      .flat_map(|centre| steps(width).map(move |width| (centre, width)))
      .find(|(centre, width)| -0.5 * width + centre == min && 0.5 * width + centre == max)
      .unwrap_or((centre, width))
  }

  /// How many single-precision steps either way [`Self::fit_span`] tries.
  const FIT_STEPS: usize = 16;

  /// `level.ltx` of a scene of `options`, its map over `bounds`' x and z (the level-bound shape's box, else the box of
  /// the placed objects), and `sectors` in the order the scene numbers them, a line at a time as `IWriter::w_string`
  /// ends them.
  ///
  /// The build options' text goes in as `w_stringZ` writes it, ended by a zero byte rather than a line break, so its
  /// last line runs on into the `;` written after it, where a reader of the line stops at the zero.
  pub fn export(options: &EditableLevelOptions, bounds: &Aabb, sectors: &[EditableSectorObject]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();

    Self::line(&mut bytes, "[map_usage]");
    Self::line(&mut bytes, &format!("ver={}", options.map_version));

    for (bit, name) in Self::GAME_TYPES {
      if options.game_type & bit != 0 {
        Self::line(&mut bytes, name);
      }
    }

    Self::line(&mut bytes, ";");
    Self::line(&mut bytes, "[level_map]");
    Self::line(
      &mut bytes,
      &format!(
        "bound_rect = {},{},{},{}",
        EditableLevelValue::format_f32(bounds.min.x),
        EditableLevelValue::format_f32(bounds.min.z),
        EditableLevelValue::format_f32(bounds.max.x),
        EditableLevelValue::format_f32(bounds.max.z)
      ),
    );
    Self::line(
      &mut bytes,
      &format!("texture = {}", Self::get_texture(&options.level_path)),
    );
    Self::line(&mut bytes, ";");

    if !options.build_options.is_empty() {
      bytes.extend_from_slice(options.build_options.as_bytes());
      bytes.push(0);
    }

    Self::line(&mut bytes, ";");
    Self::line(&mut bytes, "[sub_level_map]");

    for (number, sector) in sectors.iter().enumerate() {
      Self::line(&mut bytes, &format!("{number} = {}", sector.map_index));
    }

    bytes
  }

  /// The map's texture the Build names for a level built into `level_path`.
  pub fn get_texture(level_path: &str) -> String {
    format!("map\\map_{level_path}")
  }

  fn read_bounds(value: &str) -> XrfResult<[f32; 4]> {
    let floats: Vec<f32> = value
      .split(',')
      .map(|float| float.trim().parse::<f32>())
      .collect::<Result<_, _>>()
      .map_err(|error| XrfError::new_parsing_error(format!("bound_rect '{value}' is not four floats: {error}")))?;

    floats
      .try_into()
      .map_err(|_| XrfError::new_parsing_error(format!("bound_rect '{value}' is not four floats")))
  }

  /// Writes `text` as `IWriter::w_string` does: the text, then a line break.
  fn line(bytes: &mut Vec<u8>, text: &str) {
    bytes.extend_from_slice(text.as_bytes());
    bytes.extend_from_slice(b"\r\n");
  }
}
