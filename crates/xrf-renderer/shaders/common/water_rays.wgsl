#import "common/camera"

// The record a water pixel leaves the scene pass while every pixel traces every frame (`WaterRays`), in place of a
// request: four words at its drawn place, its ray's direction, octahedral as two snorm halves; what the screen
// answered, its colour already weighed by its share then the share (`trace_request_screen`'s two words); and the
// distance along the view of what the screen met as a half beside the low half of the frame it was written on, so a
// record no pixel wrote this frame is passed over without the records being cleared. The ray's start is read back
// from the water's own depth at the pixel.
const WATER_RAY_WORDS: u32 = 4u;

// Metres a water's traced ray starts off the water along its normal, so its own surface's triangles beneath do not
// meet it.
const TRACED_LIFT: f32 = 0.05;

// Where a drawn pixel's record starts.
fn water_ray_at(pixel: vec2<u32>) -> u32 {
  return (pixel.y * u32(camera.viewport.x) + pixel.x) * WATER_RAY_WORDS;
}

// A record's last word: the screen's distance as a half, and the low half of `frame`.
fn water_ray_last(screen_length: f32, frame: u32) -> u32 {
  return (pack2x16float(vec2<f32>(screen_length, 0.0)) & 0xffffu) | (frame << 16u);
}

// Whether a record's last word was written on `frame`.
fn is_water_ray_of(last: u32, frame: u32) -> bool {
  return (last >> 16u) == (frame & 0xffffu);
}
