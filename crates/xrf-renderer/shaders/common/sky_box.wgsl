// The half box `RenderSky` draws a sky cube through, which every pass reading a sky turns a direction by, and the
// direction every other cube is looked up by.

// Where `hbox_verts` puts the ring the box's sides fold at, just under the horizon, and how far below the bottom face
// the box's lower half samples, so all of it reads the cube's bottom rim.
const BOX_HORIZON: f32 = -0.01;
const BOX_BOTTOM: f32 = -1.01;

// A renderer-space direction as the engine looks a cube up by it: in engine space, the `z` the packer negated negated
// back, and never turned. `sky_rotation` turns the sky box's transform alone (`dxEnvironmentRender`'s
// `mSky.rotateY`); water's `s_env0`/`s_env1` and a model's `s_env` sample their cubes by the plain world direction in
// both engines (`water.ps`, Anomaly's `calc_envmap`, `model_env_lq`).
fn cube_lookup(direction: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(direction.x, direction.y, -direction.z);
}

// A renderer-space direction in the sky box's own axes: the cube lookup, turned back by the `rotateY(sky_rotation)` the
// box is drawn with.
fn sky_box_direction(direction: vec3<f32>, rotation: f32) -> vec3<f32> {
  let engine: vec3<f32> = cube_lookup(direction);
  let x: f32 = engine.x;
  let z: f32 = engine.z;
  let c: f32 = cos(rotation);
  let s: f32 = sin(rotation);

  return vec3<f32>(x * c - z * s, direction.y, x * s + z * c);
}

// The cube coordinate `hbox_verts` gives a direction: the top face as it is, each side's whole height folded into the
// sky above the horizon, and everything below it reading the bottom rim, which is the haze a cube is painted with.
fn sky_box_lookup(box: vec3<f32>) -> vec3<f32> {
  let side: f32 = max(abs(box.x), abs(box.z));
  let height: f32 = box.y / side;
  // Each side's two bands, straight across in the face's plane as its vertices interpolate.
  let upper: f32 = -1.0 + (height - BOX_HORIZON) * (2.0 / (1.0 - BOX_HORIZON));
  let lower: f32 = BOX_BOTTOM + (height + 1.0) * ((-1.0 - BOX_BOTTOM) / (1.0 + BOX_HORIZON));
  let on_side: vec3<f32> = vec3<f32>(box.x / side, select(lower, upper, height >= BOX_HORIZON), box.z / side);
  let on_bottom: vec3<f32> = vec3<f32>(box.x / -box.y, BOX_BOTTOM, box.z / -box.y);

  return select(select(on_side, on_bottom, -box.y > side), box, box.y > side);
}
