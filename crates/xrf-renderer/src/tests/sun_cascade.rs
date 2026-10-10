use glam::{Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::lighting::sun_cascade::SunCascade;
use crate::lighting::sun_view_rays::SunViewRays;

const DOWN: Vec3 = Vec3::new(0.0, -1.0, 0.0);

/// A level standing four hundred metres either way along the light from the origin.
const SPAN: Option<(f32, f32)> = Some((-400.0, 400.0));

/// A camera at a point looking at another, the up axis taken as `y` unless it looks along it.
fn create_camera(position: Vec3, target: Vec3) -> CameraView {
  let direction: Vec3 = (target - position).normalize();
  let up: Vec3 = if direction.dot(Vec3::Y).abs() > 0.99 {
    Vec3::Z
  } else {
    Vec3::Y
  };

  CameraView::new(
    position,
    glam::camera::rh::view::look_at_mat4(position, target, up),
    67.5,
    1.7,
    0.2,
    5000.0,
  )
}

/// A camera at a point looking along `+z`.
fn create_forward_camera(position: Vec3) -> CameraView {
  create_camera(position, position + Vec3::Z)
}

fn is_inside(cascade: &SunCascade, point: Vec3) -> bool {
  cascade
    .view
    .get_planes()
    .iter()
    .all(|plane: &Vec4| plane.truncate().dot(point) + plane.w >= -1e-4)
}

/// The engine's three cascades, placed one after another along the camera's view.
fn fit_chain(camera: &CameraView, light: Vec3) -> Vec<SunCascade> {
  let mut rays: SunViewRays = SunViewRays::new(camera);

  [20.0, 40.0, 160.0]
    .iter()
    .map(|width| {
      let mut cascade: SunCascade = SunCascade::default();

      cascade.fit(camera, &mut rays, light, *width, 2048, SPAN);

      cascade
    })
    .collect()
}

fn fit_first(camera: &CameraView, light: Vec3, width: f32) -> SunCascade {
  let mut rays: SunViewRays = SunViewRays::new(camera);
  let mut cascade: SunCascade = SunCascade::default();

  cascade.fit(camera, &mut rays, light, width, 2048, SPAN);

  cascade
}

#[test]
fn sun_cascade_brings_its_back_edge_to_the_near_plane_and_covers_what_is_ahead() {
  let cascade: SunCascade = fit_first(&create_forward_camera(Vec3::new(0.0, 1.7, 0.0)), DOWN, 20.0);

  assert!(is_inside(&cascade, Vec3::new(0.0, 0.0, 1.0)));
  assert!(is_inside(&cascade, Vec3::new(0.0, 0.0, 19.0)));
  assert!(!is_inside(&cascade, Vec3::new(0.0, 0.0, -1.0)));
  assert!(!is_inside(&cascade, Vec3::new(0.0, 0.0, 21.0)));
  assert!(is_inside(&cascade, Vec3::new(9.0, 0.0, 10.0)));
  assert!(!is_inside(&cascade, Vec3::new(11.0, 0.0, 10.0)));
  assert!((cascade.texel - 20.0 / 2048.0).abs() < 1e-6);
}

#[test]
fn sun_cascade_starts_where_the_views_edges_leave_the_one_before() {
  let chain: Vec<SunCascade> = fit_chain(&create_forward_camera(Vec3::new(0.0, 1.7, 0.0)), DOWN);

  assert!(is_inside(&chain[0], Vec3::new(0.0, 0.0, 19.0)));
  assert!(is_inside(&chain[1], Vec3::new(0.0, 0.0, 38.0)));
  assert!(!is_inside(&chain[1], Vec3::new(0.0, 0.0, 41.0)));
  assert!(is_inside(&chain[2], Vec3::new(0.0, 0.0, 150.0)));
  assert!(!is_inside(&chain[2], Vec3::new(0.0, 0.0, 160.0)));
}

// Looking down with the sun ahead, an edge runs back out through the back of every square the engine brings up to
// where it starts, and the ground nearest the camera was in none of them.
#[test]
fn sun_cascade_holds_the_ground_nearest_the_camera_in_every_cascade() {
  let position: Vec3 = Vec3::new(0.0, 3.8, 0.0);
  let camera: CameraView = create_camera(position, Vec3::new(0.0, 3.8 - 50f32.to_radians().tan(), 1.0));
  let chain: Vec<SunCascade> = fit_chain(&camera, Vec3::new(0.0, -0.5, -0.866).normalize());

  for ray in SunViewRays::new(&camera).near {
    let t: f32 = -ray.origin.y / ray.direction.y;

    if t > 0.0 && t < 10.0 {
      let ground: Vec3 = ray.origin + ray.direction * t;

      assert!(chain.iter().all(|cascade| is_inside(cascade, ground)), "{ground}");
    }
  }
}

#[test]
fn sun_cascade_takes_casters_and_receivers_from_as_far_as_the_level_stands() {
  let cascade: SunCascade = fit_first(&create_forward_camera(Vec3::new(0.0, 1.7, 0.0)), DOWN, 20.0);

  assert!(is_inside(&cascade, Vec3::new(0.0, 399.0, 10.0)));
  assert!(!is_inside(&cascade, Vec3::new(0.0, 420.0, 10.0)));
  assert!(is_inside(&cascade, Vec3::new(0.0, -399.0, 10.0)));
  assert!(!is_inside(&cascade, Vec3::new(0.0, -440.0, 10.0)));
}

// A map that slid with every centimetre the camera moves shimmers along every shadow's edge.
#[test]
fn sun_cascade_moves_a_whole_texel_at_a_time_and_says_so_only_when_it_does() {
  let mut cascade: SunCascade = SunCascade::default();
  let fit = |x: f32, cascade: &mut SunCascade| {
    let camera: CameraView = create_forward_camera(Vec3::new(x, 1.7, 0.0));
    let mut rays: SunViewRays = SunViewRays::new(&camera);

    cascade.fit(&camera, &mut rays, DOWN, 20.0, 2048, SPAN);
  };

  fit(0.0, &mut cascade);

  let (version, position): (u64, Vec3) = (cascade.version, cascade.view.position);

  fit(0.001, &mut cascade);
  assert_eq!(cascade.version, version);
  assert_eq!(cascade.view.position, position);

  fit(0.5, &mut cascade);
  assert_eq!(cascade.version, version + 1);

  let texels: f32 = cascade.view.position.x / cascade.texel;

  assert!((texels - texels.round()).abs() < 1e-3);
}

// Along an oblique light, a camera moving across the level moves along the light too, which moves no texel.
#[test]
fn sun_cascade_keeps_its_place_along_an_oblique_light_to_a_step() {
  let light: Vec3 = Vec3::new(1.0, -2.0, 0.5).normalize();
  let mut cascade: SunCascade = SunCascade::default();
  let fit = |along: f32, cascade: &mut SunCascade| {
    let camera: CameraView = create_forward_camera(light * along);
    let mut rays: SunViewRays = SunViewRays::new(&camera);

    cascade.fit(&camera, &mut rays, light, 20.0, 2048, SPAN);
  };

  fit(0.0, &mut cascade);

  let (version, position): (u64, Vec3) = (cascade.version, cascade.view.position);

  fit(0.5, &mut cascade);
  assert_eq!(cascade.version, version);
  assert_eq!(cascade.view.position, position);

  fit(2.0, &mut cascade);
  assert_eq!(cascade.version, version + 1);
  assert!(is_inside(&cascade, light * 2.0));
}

#[test]
fn sun_cascade_stays_over_the_camera_looking_along_the_light() {
  let cascade: SunCascade = fit_first(&create_camera(Vec3::new(0.0, 10.0, 0.0), Vec3::ZERO), DOWN, 20.0);

  assert!(is_inside(&cascade, Vec3::new(9.0, 0.0, 9.0)));
  assert!(is_inside(&cascade, Vec3::new(-9.0, 0.0, -9.0)));
}

#[test]
fn sun_cascade_draws_from_the_suns_side_looking_along_its_light() {
  let light: Vec3 = Vec3::new(1.0, -2.0, 0.5).normalize();
  let cascade: SunCascade = fit_first(&create_forward_camera(Vec3::new(0.0, 10.0, 0.0)), light, 40.0);
  let looking: Vec3 = -cascade.view.view.inverse().z_axis.truncate();

  assert!((looking.dot(light) - 1.0).abs() < 1e-4);
  assert!((2.0 / cascade.view.projection.x_axis.x - 40.0).abs() < 1e-3);
}

// A staggered map waits for its turn while the camera walks, and draws at once when a turn leaves its view outside it.
#[test]
fn sun_cascade_is_held_by_its_map_through_a_walk_but_not_a_sharp_turn() {
  let position: Vec3 = Vec3::new(0.0, 1.7, 0.0);
  let drawn: SunCascade = fit_first(&create_forward_camera(position), DOWN, 160.0);

  assert!(fit_first(&create_forward_camera(position + Vec3::new(0.5, 0.0, 1.5)), DOWN, 160.0).is_held_by(&drawn));
  assert!(!fit_first(&create_camera(position, position + Vec3::X), DOWN, 160.0).is_held_by(&drawn));
  assert!(!fit_first(&create_camera(position, position - Vec3::Z), DOWN, 160.0).is_held_by(&drawn));
}

// A running clock turns the sun a little every frame: a map waits for its turn through a turn moving a caster at the
// reach half a texel, and draws at once past it.
#[test]
fn sun_cascade_is_held_by_its_map_through_a_clocks_turn_but_not_a_seek() {
  let camera: CameraView = create_forward_camera(Vec3::new(0.0, 1.7, 0.0));
  let light: Vec3 = Vec3::new(0.3, -1.0, 0.2).normalize();
  let turned = |angle: f32| glam::Quat::from_rotation_z(angle) * light;
  let drawn: SunCascade = fit_first(&camera, light, 160.0);
  // Half a texel of 160 m at 2048 across, over the level's four hundred metres towards the sun.
  let limit: f32 = 0.5 * 160.0 / 2048.0 / 400.0;

  assert!(fit_first(&camera, turned(limit * 0.5), 160.0).is_held_by(&drawn));
  assert!(!fit_first(&camera, turned(limit * 2.0), 160.0).is_held_by(&drawn));
  assert!(!fit_first(&camera, turned(0.01), 160.0).is_held_by(&drawn));
}

// A low sun's caster far across the level shadows the ground the camera looks at, however far off it stands.
#[test]
fn sun_cascade_holds_every_caster_the_level_stands_towards_the_sun() {
  let light: Vec3 = Vec3::new(1.0, -0.08, 0.3).normalize();
  let ground: Vec3 = Vec3::new(0.0, 0.0, 10.0);
  let caster: Vec3 = ground - light * 1200.0;
  let span: Option<(f32, f32)> = Some((caster.dot(light) - 1.0, ground.dot(light) + 50.0));

  for height in [1.7, 30.0] {
    let camera: CameraView = create_forward_camera(Vec3::new(0.0, height, 0.0));
    let mut rays: SunViewRays = SunViewRays::new(&camera);
    let mut cascade: SunCascade = SunCascade::default();

    cascade.fit(&camera, &mut rays, light, 160.0, 2048, span);

    assert!(is_inside(&cascade, ground));
    assert!(is_inside(&cascade, caster));
  }
}

#[test]
fn sun_cascade_is_not_held_by_a_map_drawn_over_another_level() {
  let camera: CameraView = create_forward_camera(Vec3::new(0.0, 1.7, 0.0));
  let drawn: SunCascade = fit_first(&camera, DOWN, 160.0);
  let mut rays: SunViewRays = SunViewRays::new(&camera);
  let mut wider: SunCascade = SunCascade::default();

  wider.fit(&camera, &mut rays, DOWN, 160.0, 2048, Some((-600.0, 400.0)));

  assert!(!wider.is_held_by(&drawn));
}

#[test]
fn sun_cascade_is_not_held_by_a_map_drawn_at_other_settings() {
  let camera: CameraView = create_forward_camera(Vec3::new(0.0, 1.7, 0.0));
  let drawn: SunCascade = fit_first(&camera, DOWN, 160.0);
  let mut rays: SunViewRays = SunViewRays::new(&camera);
  let mut finer: SunCascade = SunCascade::default();

  finer.fit(&camera, &mut rays, DOWN, 160.0, 4096, SPAN);

  assert!(fit_first(&camera, DOWN, 160.0).is_held_by(&drawn));
  assert!(!finer.is_held_by(&drawn));
  assert!(!fit_first(&camera, Vec3::new(0.3, -1.0, 0.0).normalize(), 160.0).is_held_by(&drawn));
  assert!(!drawn.is_held_by(&SunCascade::default()));
}

/// Where a point lands on a map `resolution` texels across drawn by `view` into the rows from `first`: its texel's
/// column and row, and its depth.
fn to_texel(view: &CameraView, point: Vec3, (first, rows): (u32, u32), resolution: u32) -> Vec3 {
  let clip: Vec4 = view.get_view_projection() * point.extend(1.0);
  let ndc: Vec3 = clip.truncate() / clip.w;

  Vec3::new(
    (ndc.x + 1.0) / 2.0 * resolution as f32,
    first as f32 + (1.0 - ndc.y) / 2.0 * rows as f32,
    ndc.z,
  )
}

#[test]
fn sun_cascade_rows_land_where_the_whole_map_s_would() {
  let light: Vec3 = Vec3::new(0.3, -1.0, 0.2).normalize();
  let cascade: SunCascade = fit_first(&create_forward_camera(Vec3::new(5.0, 1.7, 3.0)), light, 160.0);
  let center: Vec3 = cascade.view.position + light * 400.0;
  let right: Vec3 = light.cross(Vec3::Y).normalize();
  let up: Vec3 = right.cross(light);
  let mut tested: usize = 0;

  for (first, last) in [(0, 512), (512, 1024), (1024, 1536), (1536, 2048)] {
    let band: CameraView = cascade.get_rows_view((first, last), 2048);

    for (across, down) in [(-60.0, -70.0), (10.0, 0.0), (75.0, 40.0), (-20.0, 79.0)] {
      let point: Vec3 = center + right * across - up * down;
      let whole: Vec3 = to_texel(&cascade.view, point, (0, 2048), 2048);

      if whole.y < first as f32 || whole.y >= last as f32 {
        continue;
      }

      let banded: Vec3 = to_texel(&band, point, (first, last - first), 2048);

      assert!(whole.distance(banded) < 1e-2, "{whole} against {banded}");
      tested += 1;
    }
  }

  assert_eq!(tested, 4);
}
