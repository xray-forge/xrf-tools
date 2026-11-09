use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};
use xrf_math::{Vector2d, Vector3d};

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// The points a fuzzy point light spreads over, `CLight::SFuzzyData`: the build makes a static light of each, sharing
/// the light's energy (`SceneBuilder::BuildPointLight`, `BuilderRemote.cpp:1034`).
#[derive(Clone, Debug, PartialEq)]
pub struct EditableLightFuzzy {
  /// `EShapeType`: 0 a sphere, 1 a box, the volume new points are spread through.
  pub shape: u8,
  pub sphere_radius: f32,
  pub box_dimensions: Vector3d,
  /// The points, in the light's own space.
  pub points: Vec<Vector3d>,
}

/// A light the scene places, `CLight`: an object of `light.part` or of a group, which the build hands the level
/// compiler as a static light, a dynamic one, or both (`SceneBuilder::BuildLight`, `BuilderRemote.cpp:1080`).
#[derive(Clone, Debug, PartialEq)]
pub struct EditableLightObject {
  pub object: EditableCustomObject,
  /// `ELight::EType`: 1 a point light, 2 a spot, 3 directional; the build exports point lights only.
  pub kind: u32,
  /// Red, green, blue and alpha; the build lights with the colour times [`Self::brightness`].
  pub color: [f32; 4],
  pub brightness: f32,
  pub range: f32,
  pub attenuation: [f32; 3],
  /// A spot's cone angle, in radians.
  pub cone: f32,
  pub virtual_size: f32,
  pub use_in_d3d: bool,
  /// [`Self::AFFECT_STATIC`], [`Self::AFFECT_DYNAMIC`], [`Self::PROCEDURAL`] and any bit the editor does not name,
  /// without [`Self::POINT_FUZZY`]: [`Self::fuzzy`] stands for it, so a fuzzy flag without points (which `LoadLTX`
  /// would fail on) cannot be written.
  pub flags: u32,
  /// The light control it answers to, by its index in the light tool's `[lcontrols]`.
  pub control: u32,
  /// The light animation it plays, empty for none.
  pub animation: String,
  pub falloff_texture: String,
  /// The points of a fuzzy point light, saved with [`Self::POINT_FUZZY`] in its flags.
  pub fuzzy: Option<EditableLightFuzzy>,
}

impl EditableLightObject {
  /// `LIGHT_VERSION`, the only one `CLight::LoadLTX` reads.
  pub const VERSION: u32 = 0x11;

  /// `ELight::flAffectStatic`: baked into the level's lightmaps.
  pub const AFFECT_STATIC: u32 = 1;
  /// `ELight::flAffectDynamic`: lighting objects at run time.
  pub const AFFECT_DYNAMIC: u32 = 1 << 1;
  /// `ELight::flProcedural`.
  pub const PROCEDURAL: u32 = 1 << 2;
  /// `ELight::flPointFuzzy`: spread over [`Self::fuzzy`]'s points.
  pub const POINT_FUZZY: u32 = 1 << 4;

  /// `$static`'s index in a new scene's light controls, baked into lightmaps by the level compiler's static layer.
  pub const STATIC_CONTROL: u32 = 0;
  /// `$hemi`'s index: the level compiler's hemisphere layer.
  pub const HEMI_CONTROL: u32 = 1;
  /// `$sun`'s index: the level compiler's sun layer.
  pub const SUN_CONTROL: u32 = 2;

  /// A new light as `CLight::Construct` makes one (`ELight.cpp:40`), standing at `object`: a white point light of
  /// brightness 1, range 8, attenuation 1, 0, 0, a cone of `PI / 8`, affecting static lighting under `$static`.
  pub fn new(object: EditableCustomObject) -> Self {
    Self {
      object,
      kind: Self::POINT,
      color: [1.0, 1.0, 1.0, 0.0],
      brightness: 1.0,
      range: 8.0,
      attenuation: [1.0, 0.0, 0.0],
      cone: std::f32::consts::FRAC_PI_8,
      virtual_size: 0.0,
      use_in_d3d: true,
      flags: Self::AFFECT_STATIC,
      control: Self::STATIC_CONTROL,
      animation: String::new(),
      falloff_texture: String::new(),
      fuzzy: None,
    }
  }

  /// `ELight::ltPoint`, the only kind the build exports.
  pub const POINT: u32 = 1;

  /// Writes the light tool's own data as a new scene's tool saves it (`ESceneLightTool::Clear`,
  /// `ESceneLightTools.cpp:27`): the controls `$static`, `$hemi` and `$sun` at 0, 1 and 2, the next control index 3, and
  /// the default sun shadow direction, -25 and 292 degrees.
  pub fn write_new_tool_data(tool_data: &mut IniFile) {
    for (name, index) in [
      ("$static", Self::STATIC_CONTROL),
      ("$hemi", Self::HEMI_CONTROL),
      ("$sun", Self::SUN_CONTROL),
    ] {
      tool_data.section_mut("lcontrols").set(name, index.to_string());
    }

    let main: &mut IniSection = tool_data.section_mut("main");

    main.set("lcontrol_last_idx", "3");
    main.set(
      "sun_shadow_dir",
      // `deg2rad` multiplies by `PI / 180` in double precision; GSC's scenes hold `-0.436332, 5.096362`.
      EditableLevelValue::format_vector2(&Vector2d::new(
        (-25.0_f64).to_radians() as f32,
        292.0_f64.to_radians() as f32,
      )),
    );
  }

  /// The flags as the editor saves them, [`Self::POINT_FUZZY`] set for a light holding fuzzy points.
  pub fn get_saved_flags(&self) -> u32 {
    if self.fuzzy.is_some() {
      self.flags | Self::POINT_FUZZY
    } else {
      self.flags
    }
  }

  fn read_fuzzy(section: &IniSection) -> XrfResult<EditableLightFuzzy> {
    let count: u32 = EditableLevelValue::read_u32(section, "fuzzy_point_count")?;

    Ok(EditableLightFuzzy {
      shape: section.get_parsed::<u8>("fuzzy_shape_type")?,
      sphere_radius: section.get_parsed::<f32>("fuzzy_sphere_radius")?,
      box_dimensions: EditableLevelValue::read_vector3(section, "fuzzy_box_dim")?,
      points: (0..count)
        .map(|index| EditableLevelValue::read_vector3(section, &format!("fuzzy_point_{index}")))
        .collect::<XrfResult<Vec<Vector3d>>>()?,
    })
  }

  fn read_color(section: &IniSection) -> XrfResult<[f32; 4]> {
    let value: &str = section.get_required("color")?;
    let channels: Vec<f32> = value
      .split(',')
      .map(|channel| channel.trim().parse::<f32>())
      .collect::<Result<Vec<f32>, _>>()
      .map_err(|error| {
        XrfError::new_parsing_error(format!(
          "[{}] color '{value}' is not four floats: {error}",
          section.name
        ))
      })?;

    channels.try_into().map_err(|channels: Vec<f32>| {
      XrfError::new_parsing_error(format!(
        "[{}] color '{value}' holds {} floats, not four",
        section.name,
        channels.len()
      ))
    })
  }
}

impl EditableToolObject for EditableLightObject {
  /// `OBJCLASS_LIGHT`.
  const CLASS_ID: i32 = 3;
  const TOOL_NAME: &'static str = "light";
  const KEEPS_FLAGS: bool = true;

  /// As `CLight::LoadLTX` reads it, refusing another version as it does; the fuzzy points only when the flags say.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    let section: &IniSection = object.section;
    let version: u32 = section.get_parsed::<u32>("version")?;

    if version != Self::VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Light [{}] is of version {version}, not {}",
        section.name,
        Self::VERSION
      )));
    }

    let flags: u32 = EditableLevelValue::read_u32(section, "light_flags")?;

    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      kind: EditableLevelValue::read_u32(section, "type")?,
      color: Self::read_color(section)?,
      brightness: section.get_parsed::<f32>("brightness")?,
      range: section.get_parsed::<f32>("range")?,
      attenuation: [
        section.get_parsed::<f32>("attenuation0")?,
        section.get_parsed::<f32>("attenuation1")?,
        section.get_parsed::<f32>("attenuation2")?,
      ],
      cone: section.get_parsed::<f32>("cone")?,
      virtual_size: section.get_parsed::<f32>("virtual_size")?,
      use_in_d3d: EditableLevelValue::read_bool(section, "use_in_d3d")?,
      flags: flags & !Self::POINT_FUZZY,
      control: EditableLevelValue::read_u32(section, "light_control")?,
      animation: section.get_required("anim_ref_name")?.to_owned(),
      falloff_texture: section.get_required("fallof_texture")?.to_owned(),
      fuzzy: if flags & Self::POINT_FUZZY != 0 {
        Some(Self::read_fuzzy(section)?)
      } else {
        None
      },
    })
  }

  /// As `CLight::SaveLTX` writes it, at [`Self::VERSION`].
  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    let section: &mut IniSection = ini.section_mut(section);

    self.object.write_to_section(section);
    section.set("version", Self::VERSION.to_string());
    section.set("type", EditableLevelValue::format_u32(self.kind));
    section.set(
      "color",
      self
        .color
        .iter()
        .map(|channel| EditableLevelValue::format_f32(*channel))
        .collect::<Vec<String>>()
        .join(", "),
    );
    section.set("brightness", EditableLevelValue::format_f32(self.brightness));
    section.set("range", EditableLevelValue::format_f32(self.range));

    for (index, attenuation) in self.attenuation.iter().enumerate() {
      section.set(
        format!("attenuation{index}"),
        EditableLevelValue::format_f32(*attenuation),
      );
    }

    section.set("cone", EditableLevelValue::format_f32(self.cone));
    section.set("virtual_size", EditableLevelValue::format_f32(self.virtual_size));
    section.set("use_in_d3d", EditableLevelValue::format_bool(self.use_in_d3d));
    section.set("light_flags", EditableLevelValue::format_u32(self.get_saved_flags()));
    section.set("light_control", EditableLevelValue::format_u32(self.control));
    section.set("anim_ref_name", self.animation.clone());
    section.set("fallof_texture", self.falloff_texture.clone());

    if let Some(fuzzy) = &self.fuzzy {
      section.set("fuzzy_shape_type", fuzzy.shape.to_string());
      section.set(
        "fuzzy_sphere_radius",
        EditableLevelValue::format_f32(fuzzy.sphere_radius),
      );
      section.set(
        "fuzzy_box_dim",
        EditableLevelValue::format_vector3(&fuzzy.box_dimensions),
      );
      section.set("fuzzy_point_count", fuzzy.points.len().to_string());

      for (index, point) in fuzzy.points.iter().enumerate() {
        section.set(
          format!("fuzzy_point_{index}"),
          EditableLevelValue::format_vector3(point),
        );
      }
    }
  }
}
