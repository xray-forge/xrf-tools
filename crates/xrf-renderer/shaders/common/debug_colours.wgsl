// The colours the debug views read a measure by: a heat ramp for counts and the sampled mip, a band a stop for the
// luminance, and a tint a sun cascade.

const HEAT_BLUE: vec3<f32> = vec3<f32>(0.1, 0.25, 0.95);
const HEAT_CYAN: vec3<f32> = vec3<f32>(0.1, 0.75, 0.9);
const HEAT_GREEN: vec3<f32> = vec3<f32>(0.15, 0.8, 0.2);
const HEAT_YELLOW: vec3<f32> = vec3<f32>(0.95, 0.85, 0.1);
const HEAT_RED: vec3<f32> = vec3<f32>(0.9, 0.1, 0.1);

// Blue at nought through cyan, green and yellow to red at one.
fn heat_colour(share: f32) -> vec3<f32> {
  let step: f32 = saturate(share) * 4.0;

  if (step < 1.0) {
    return mix(HEAT_BLUE, HEAT_CYAN, step);
  }

  if (step < 2.0) {
    return mix(HEAT_CYAN, HEAT_GREEN, step - 1.0);
  }

  if (step < 3.0) {
    return mix(HEAT_GREEN, HEAT_YELLOW, step - 2.0);
  }

  return mix(HEAT_YELLOW, HEAT_RED, step - 3.0);
}

// A whole count: black for none, then one heat step each from blue at one to red at `most`.
fn count_colour(count: f32, most: f32) -> vec3<f32> {
  let whole: f32 = round(count);

  return select(heat_colour((whole - 1.0) / max(most - 1.0, 1.0)), vec3<f32>(0.0), whole < 0.5);
}

// The mip a texture is read at: green at its own size, blue two mips magnified past it, red two mips smaller.
fn mip_colour(mip: f32) -> vec3<f32> {
  return heat_colour((mip + 2.0) / 4.0);
}

// The luminance's band, in whole stops about middle grey: black six and more under, then purple and blues, grey at
// middle grey, greens, yellow, orange and red, white six and more over.
fn stop_colour(stops: f32) -> vec3<f32> {
  switch (i32(clamp(round(stops), -6.0, 6.0))) {
    case -6: {
      return vec3<f32>(0.0);
    }
    case -5: {
      return vec3<f32>(0.25, 0.0, 0.35);
    }
    case -4: {
      return vec3<f32>(0.2, 0.1, 0.7);
    }
    case -3: {
      return vec3<f32>(0.1, 0.3, 0.9);
    }
    case -2: {
      return vec3<f32>(0.1, 0.6, 0.9);
    }
    case -1: {
      return vec3<f32>(0.1, 0.6, 0.5);
    }
    case 0: {
      return vec3<f32>(0.5);
    }
    case 1: {
      return vec3<f32>(0.3, 0.75, 0.2);
    }
    case 2: {
      return vec3<f32>(0.65, 0.85, 0.1);
    }
    case 3: {
      return vec3<f32>(0.95, 0.85, 0.1);
    }
    case 4: {
      return vec3<f32>(1.0, 0.55, 0.1);
    }
    case 5: {
      return vec3<f32>(0.9, 0.1, 0.1);
    }
    default: {
      return vec3<f32>(1.0);
    }
  }
}

// A sun cascade's tint by its order from one, the nearest first; grey for none.
fn cascade_colour(order: f32) -> vec3<f32> {
  switch (u32(round(order))) {
    case 1u: {
      return vec3<f32>(0.9, 0.35, 0.35);
    }
    case 2u: {
      return vec3<f32>(0.35, 0.8, 0.35);
    }
    case 3u: {
      return vec3<f32>(0.35, 0.5, 0.95);
    }
    case 4u: {
      return vec3<f32>(0.9, 0.8, 0.3);
    }
    default: {
      return vec3<f32>(0.4);
    }
  }
}
