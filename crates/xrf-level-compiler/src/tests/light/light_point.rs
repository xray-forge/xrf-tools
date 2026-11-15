//! What `LightPoint` gathers at one point, by each compiler's variant.

use glam::Vec3;
use xrf_geometry::{BvhTracer, Triangle};
use xrf_level::LevelLightRole;

use crate::light::point::light_color::LightColor;
use crate::light::point::light_point::LightPoint;
use crate::light::point::light_point_variant::LightPointVariant;
use crate::light::point::light_sample_point::LightSamplePoint;
use crate::light::source::light_source::LightSource;
use crate::light::source::light_sources::LightSources;
use crate::shadow::shadow_materials::ShadowMaterials;

const DETAILS: LightPoint = LightPoint {
  variant: LightPointVariant::Details,
};

const LEVEL: LightPoint = LightPoint {
  variant: LightPointVariant::Level,
};

/// A static point light 2 m above the origin.
fn lamp() -> LightSource {
  LightSource {
    is_directional: false,
    is_point: true,
    diffuse: Vec3::new(1.0, 0.5, 0.25),
    position: Vec3::new(0.0, 2.0, 0.0),
    direction: Vec3::NEG_Y,
    range: 4.0,
    range_squared: 16.0,
    attenuation: [1.0, 0.5, 0.0],
    falloff: 0.05,
    energy: 1.0,
  }
}

/// A sun straight overhead.
fn sun() -> LightSource {
  LightSource {
    is_directional: true,
    is_point: false,
    diffuse: Vec3::ONE,
    position: Vec3::ZERO,
    direction: Vec3::NEG_Y,
    range: 0.0,
    range_squared: 0.0,
    attenuation: [1.0, 0.0, 0.0],
    falloff: 0.0,
    energy: 0.75,
  }
}

/// A square 1 m above the origin, wound so its front faces `up` or down.
fn roof(up: bool) -> BvhTracer {
  let [a, b, c, d] = [
    Vec3::new(-5.0, 1.0, -5.0),
    Vec3::new(5.0, 1.0, -5.0),
    Vec3::new(5.0, 1.0, 5.0),
    Vec3::new(-5.0, 1.0, 5.0),
  ];
  let triangles: Vec<Triangle> = if up {
    vec![Triangle::new([a, d, c]), Triangle::new([a, c, b])]
  } else {
    vec![Triangle::new([a, b, c]), Triangle::new([a, c, d])]
  };

  BvhTracer::of_triangles(triangles)
}

/// A tracer meeting nothing near the origin.
fn open() -> BvhTracer {
  BvhTracer::of_triangles(vec![Triangle::new([
    Vec3::new(100.0, 0.0, 100.0),
    Vec3::new(101.0, 0.0, 100.0),
    Vec3::new(100.0, 0.0, 101.0),
  ])])
}

/// `light` alone, a static light.
fn statics(light: LightSource) -> LightSources {
  LightSources {
    rgb: vec![light],
    ..LightSources::default()
  }
}

/// `light` alone, a sun.
fn suns(light: LightSource) -> LightSources {
  LightSources {
    sun: vec![light],
    ..LightSources::default()
  }
}

fn gather(light_point: &LightPoint, tracer: &BvhTracer, lights: LightSources) -> LightColor {
  light_point.evaluate(
    &LightSamplePoint::new(Vec3::ZERO, Vec3::Y),
    &lights,
    &LevelLightRole::ALL,
    tracer,
    &ShadowMaterials::new_opaque(2),
  )
}

#[test]
fn a_static_point_light_fades_by_each_compilers_rule() {
  let details: LightColor = gather(&DETAILS, &open(), statics(lamp()));
  let level: LightColor = gather(&LEVEL, &open(), statics(lamp()));

  // 2 m away, straight above: attenuation 1 + 0.5 * 2 = 2.
  assert_eq!(
    details.rgb,
    Vec3::new(1.0, 0.5, 0.25) * 0.5,
    "xrDO_Light: over the attenuation"
  );
  assert_eq!(
    level.rgb,
    Vec3::new(1.0, 0.5, 0.25) * (0.5 - 2.0 * 0.05),
    "xrLC: the attenuation's inverse less the falloff at that distance"
  );
}

#[test]
fn the_sun_gives_its_energy_and_a_light_behind_gives_nothing() {
  let lit: LightColor = gather(&LEVEL, &open(), suns(sun()));
  let behind: LightColor = gather(
    &LEVEL,
    &open(),
    suns(LightSource {
      direction: Vec3::Y,
      ..sun()
    }),
  );

  assert_eq!(lit.sun, 0.75, "no cosine for the sun");
  assert_eq!(behind.sun, 0.0);
}

#[test]
fn xrlc_shadows_from_either_side_of_a_face_and_xrdo_light_from_its_front() {
  // The roof's front faces down, toward the point: both compilers' rays meet it.
  for light_point in [DETAILS, LEVEL] {
    assert_eq!(gather(&light_point, &roof(false), suns(sun())).sun, 0.0);
  }

  // Its front faces up, away from the point: xrDO_Light's ray passes its back, xrLC's does not.
  assert_eq!(gather(&DETAILS, &roof(true), suns(sun())).sun, 0.75);
  assert_eq!(gather(&LEVEL, &roof(true), suns(sun())).sun, 0.0);
}
