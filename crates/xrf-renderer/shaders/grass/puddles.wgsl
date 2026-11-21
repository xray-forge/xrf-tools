#import "common/puddle_site"
#import "grass/planting_bindings"

// Metres past the band a site holds water over a tuft's foot may stand and still be in its puddle.
const TUFT_SLACK: f32 = 0.15;

// The share of its size a tuft stands at by the puddles this frame, its foot `world`, in renderer space as the
// wetting reads its sites, `from_eye` metres from the eye: by how far out from the nearest puddle's site it stands
// (`site_reach`), as far as the wetting draws that puddle (`puddle_drawn`, `puddle_tuft`).
fn puddle_kept(world: vec3<f32>, from_eye: f32) -> f32 {
  let drawn: f32 = puddle_drawn(from_eye, puddle_window.state.y);

  if (puddle_window.state.x < 0.5 || drawn <= 0.0) {
    return 1.0;
  }

  return puddle_tuft(site_reach(puddles, puddle_window.surface, puddle_window.shape.z, world, TUFT_SLACK), drawn);
}
