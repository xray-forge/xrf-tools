use xrf_editable_level::{
  EditableCustomObject, EditableLevelValue, EditableLightObject, EditableObjectPart, EditableObjectTransform,
};
use xrf_level::{LevelDynamicLight, LevelFile, LevelLight, LevelLightsFile};
use xrf_math::{Vector2d, Vector3d};

use crate::level::level_decompile_context::LevelDecompileContext;

/// A level's authored lights, from the compiled `build.lights` and the bundle's `fsL_LIGHT_DYNAMIC` back into the
/// scene's `light.part` (decision 15).
///
/// The build hands each light to the level compiler as a static light, a dynamic one or both
/// (`SceneBuilder::BuildLight`, `BuilderRemote.cpp:1080`). The compiler writes the static ones into `build.lights` by
/// their control's layer, chunk 0 for `$static` and 1 for `$hemi` beside the sky it generates, each diffuse divided by
/// its energy and its range times 1.1; the dynamic ones into `fsL_LIGHT_DYNAMIC` as given. Neither keeps a light's
/// colour and brightness apart, its fuzzy points, or a dynamic light's control.
pub struct LevelLightConversion;

impl LevelLightConversion {
  /// The `build.lights` chunks of the authored layers, and the control each was built under.
  const LAYERS: [(u32, u32); 2] = [
    (0, EditableLightObject::STATIC_CONTROL),
    (1, EditableLightObject::HEMI_CONTROL),
  ];

  /// How far a static light's diffuse, divided and multiplied back by its energy, may stand from its dynamic twin's.
  const DIFFUSE_TOLERANCE: f32 = 1e-5;

  /// The light part a level was built from: a light per compiled point light of an authored layer, one affecting both
  /// static and dynamic lighting where a dynamic light stands at the same position with the same diffuse, and one per
  /// remaining dynamic point light under `$static`; each named `light_NNNN`, unturned, its brightness the channel of
  /// its diffuse of the largest magnitude and its colour the rest. A fuzzy light comes back as one light per point it
  /// was spread over. The tool data is a new scene's.
  pub fn decompile(
    lights: Option<&LevelLightsFile>,
    level: &LevelFile,
    context: &mut LevelDecompileContext,
  ) -> EditableObjectPart<EditableLightObject> {
    let dynamics: Vec<&LevelDynamicLight> = level
      .lights
      .as_ref()
      .map(|chunk| {
        chunk
          .lights
          .iter()
          .filter(|light| light.kind == LevelDynamicLight::KIND_POINT)
          .collect()
      })
      .unwrap_or_default();
    let mut taken: Vec<bool> = vec![false; dynamics.len()];
    let mut objects: Vec<EditableLightObject> = Vec::new();

    for (chunk, control) in Self::LAYERS {
      let compiled = lights
        .into_iter()
        .flat_map(|file| Self::get_chunk_lights(file, chunk))
        .filter(|light| light.kind == LevelLight::KIND_POINT);

      for light in compiled {
        let diffuse: Vector3d = &light.diffuse * light.energy;
        let twin: Option<usize> = (0..dynamics.len()).find(|&index| {
          !taken[index]
            && dynamics[index].position == light.position
            && Self::get_diffuse(dynamics[index]).distance(&diffuse)
              <= Self::DIFFUSE_TOLERANCE * diffuse.length().max(1.0)
        });
        let mut object: EditableLightObject = match twin {
          Some(index) => {
            taken[index] = true;

            let mut object: EditableLightObject = Self::from_dynamic(dynamics[index], context);

            object.flags |= EditableLightObject::AFFECT_STATIC;
            object
          }
          None => {
            let mut object: EditableLightObject = Self::new_light(light.position, &diffuse, context);

            object.range = Self::get_saved_range(light.range);
            object.attenuation = [
              light.attenuation_constant,
              light.attenuation_linear,
              light.attenuation_quadratic,
            ];
            object
          }
        };

        object.control = control;
        objects.push(object);
      }
    }

    for (index, dynamic) in dynamics.iter().enumerate() {
      if !taken[index] {
        objects.push(Self::from_dynamic(dynamic, context));
      }
    }

    let mut part: EditableObjectPart<EditableLightObject> = EditableObjectPart::new(context.guid, objects);

    if let Some(direction) = Self::get_sun_direction(level) {
      part.tool_data.section_mut(EditableLightObject::MAIN_SECTION).set(
        EditableLightObject::SUN_DIRECTION_KEY,
        EditableLevelValue::format_vector2(&direction),
      );
    }

    part
  }

  /// The level options' hemisphere quality the build generated `build.lights` chunk 1's sky with
  /// (`SceneBuilder::BuildHemiLights`, `BuilderRemote.cpp:946`): one light for 0, else `xrHemisphereBuild`'s 26, 91 or
  /// 196 for 1 to 3. None for a sky of another count.
  pub fn get_hemi_quality(lights: &LevelLightsFile) -> Option<u8> {
    let sky: usize = Self::get_chunk_lights(lights, 1)
      .filter(|light| light.kind != LevelLight::KIND_POINT)
      .count();

    [1, 26, 91, 196]
      .into_iter()
      .position(|count| count == sky)
      .map(|quality| quality as u8)
  }

  /// The level options' sun quality the build spread chunk 2's sun with (`SceneBuilder::BuildSun`,
  /// `BuilderRemote.cpp:981`): 1, 3, 4 or 7 samples a side for 0 to 3. None for a sun of another count.
  pub fn get_sun_quality(lights: &LevelLightsFile) -> Option<u8> {
    let sun: usize = Self::get_chunk_lights(lights, 2).count();

    [1, 9, 16, 49]
      .into_iter()
      .position(|count| count == sun)
      .map(|quality| quality as u8)
  }

  /// The light tool's `sun_shadow_dir`, pitch then heading, that the build pointed the dynamic sun along
  /// (`direction.setHP(dir.y, dir.x)`), its heading within a turn upwards of 0 as the editor's default 292 degrees is.
  fn get_sun_direction(level: &LevelFile) -> Option<Vector2d> {
    let sun: &LevelDynamicLight = level.lights.as_ref()?.lights.iter().find(|light| light.is_sun())?;
    let heading: f32 = (-sun.direction.x).atan2(sun.direction.z);

    Some(Vector2d::new(
      sun.direction.y.clamp(-1.0, 1.0).asin(),
      if heading < 0.0 {
        heading + std::f32::consts::TAU
      } else {
        heading
      },
    ))
  }

  fn get_chunk_lights(lights: &LevelLightsFile, chunk: u32) -> impl Iterator<Item = &LevelLight> {
    lights
      .chunks
      .iter()
      .filter(move |layer| layer.get_id() == chunk)
      .flat_map(|layer| layer.get_lights())
  }

  fn get_diffuse(light: &LevelDynamicLight) -> Vector3d {
    Vector3d::new(light.diffuse.r, light.diffuse.g, light.diffuse.b)
  }

  /// A dynamic point light as the build was given it: its diffuse, range, attenuation and cone, under `$static`.
  fn from_dynamic(light: &LevelDynamicLight, context: &mut LevelDecompileContext) -> EditableLightObject {
    let mut object: EditableLightObject = Self::new_light(light.position, &Self::get_diffuse(light), context);

    object.flags = EditableLightObject::AFFECT_DYNAMIC;
    object.range = light.range;
    object.attenuation = [light.attenuation_0, light.attenuation_1, light.attenuation_2];
    object.cone = light.phi;
    object
  }

  /// A new light at `position` lighting with `diffuse`: brightness its channel of the largest magnitude, sign kept, so
  /// a darkening light of negative channels stays one, and colour the diffuse over it.
  fn new_light(position: Vector3d, diffuse: &Vector3d, context: &mut LevelDecompileContext) -> EditableLightObject {
    let mut object: EditableLightObject = EditableLightObject::new(EditableCustomObject::new(
      context.names.get_next("light"),
      EditableObjectTransform {
        position,
        rotation: Vector3d::ZERO,
        scale: Vector3d::new(1.0, 1.0, 1.0),
      },
    ));
    let brightness: f32 = [diffuse.x, diffuse.y, diffuse.z]
      .into_iter()
      .fold(0.0, |strongest: f32, channel| {
        if channel.abs() > strongest.abs() {
          channel
        } else {
          strongest
        }
      });

    if brightness == 0.0 {
      object.brightness = 0.0;
    } else {
      object.brightness = brightness;
      object.color = [
        diffuse.x / brightness,
        diffuse.y / brightness,
        diffuse.z / brightness,
        0.0,
      ];
    }

    object
  }

  /// The range a static light was authored at, whose 1.1 times is the compiled one: the nearest float to the quotient
  /// whose product gives the compiled bits, else the quotient.
  fn get_saved_range(compiled: f32) -> f32 {
    let quotient: f32 = compiled / 1.1;

    [0, 1, -1, 2, -2]
      .into_iter()
      .map(|step: i32| f32::from_bits(quotient.to_bits().wrapping_add_signed(step)))
      .find(|range| (range * 1.1).to_bits() == compiled.to_bits())
      .unwrap_or(quotient)
  }
}
